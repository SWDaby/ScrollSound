//! Elevation: admin-token detection and elevated/unelevated relaunch (design
//! D7, spec: `elevation`). Runs before any UI; the launch gates live in
//! `main.rs` / `taskbar_menu.rs`.

use std::mem::size_of;
use std::ptr;

use windows::core::{BOOL, PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HANDLE};
use windows::Win32::Security::{
    AllocateAndInitializeSid, CheckTokenMembership, DuplicateTokenEx, FreeSid,
    SecurityImpersonation, TokenPrimary, PSID, SID_IDENTIFIER_AUTHORITY, TOKEN_ALL_ACCESS,
    TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_QUERY,
};
use windows::Win32::System::Threading::{
    CreateProcessWithTokenW, OpenProcess, OpenProcessToken, PROCESS_CREATION_FLAGS,
    PROCESS_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, STARTUPINFOW,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};
use windows::Win32::UI::WindowsAndMessaging::{GetShellWindow, GetWindowThreadProcessId};

use crate::platform::win_settings;

// Values from winnt.h/winbase.h, kept local (plain constants).
const SECURITY_NT_AUTHORITY: [u8; 6] = [0, 0, 0, 0, 0, 5];
const SECURITY_BUILTIN_DOMAIN_RID: u32 = 0x20;
const DOMAIN_ALIAS_RID_ADMINS: u32 = 0x220;

/// Whether the given token (`None` = current process/thread token) has the
/// Administrators group enabled.
fn is_admin_token(token: Option<HANDLE>) -> bool {
    let authority = SID_IDENTIFIER_AUTHORITY {
        Value: SECURITY_NT_AUTHORITY,
    };
    let mut sid: PSID = PSID(std::ptr::null_mut());
    let allocated = unsafe {
        AllocateAndInitializeSid(
            &authority,
            2,
            SECURITY_BUILTIN_DOMAIN_RID,
            DOMAIN_ALIAS_RID_ADMINS,
            0,
            0,
            0,
            0,
            0,
            0,
            &mut sid,
        )
    };
    if allocated.is_err() {
        return false;
    }
    let mut member = BOOL::default();
    let checked = unsafe { CheckTokenMembership(token, sid, &mut member) };
    unsafe {
        FreeSid(sid);
    }
    checked.is_ok() && member.as_bool()
}

/// Whether the current process token is a member of the Administrators group.
pub fn is_admin() -> bool {
    is_admin_token(None)
}

/// Re-launches the current executable via the `runas` verb (UAC prompt) with
/// the current arguments; the caller then exits. Returns whether the launch
/// was accepted (a UAC cancellation is an error, not a crash).
pub fn relaunch_elevated() -> bool {
    let Some(exe) = win_settings::exe_path() else {
        return false;
    };
    let verb = crate::platform::wide_cstr("runas");
    let file = crate::platform::wide_cstr(&exe);
    let args: Vec<String> = std::env::args().skip(1).collect();
    // Always include the relaunch marker: the elevated child must wait for
    // this (exiting) process to release the single-instance mutex.
    let mut parts = vec![crate::platform::single_instance::RELAUNCH_ARG.to_string()];
    parts.extend(args);
    let params: Vec<u16> = crate::platform::wide_cstr(&parts.join(" "));
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: if params.is_empty() {
            PCWSTR::null()
        } else {
            PCWSTR(params.as_ptr())
        },
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut sei) }.is_ok()
}

/// RAII close for the temporary handles below.
struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Re-launches the current executable with an unelevated (standard-user)
/// token borrowed from the shell (Explorer): the interactive-session path for
/// dropping elevation without a full manual restart. The caller exits after a
/// successful launch. Returns whether the child process was created.
pub fn relaunch_unelevated() -> bool {
    let Some(exe) = win_settings::exe_path() else {
        return false;
    };

    // Explorer owns the shell window and runs with the user's filtered token
    // (unelevated) on an interactive session.
    let shell = unsafe { GetShellWindow() };
    if shell.0.is_null() {
        tracing::error!("relaunch_unelevated: no shell window");
        return false;
    }
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(shell, Some(&mut pid)) };
    if pid == 0 {
        tracing::error!("relaunch_unelevated: no shell process id");
        return false;
    }
    let proc_handle = match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
        Ok(h) => h,
        Err(e) => {
            tracing::error!("relaunch_unelevated: OpenProcess failed: {e}");
            return false;
        }
    };
    let _proc = OwnedHandle(proc_handle);

    let mut token = HANDLE::default();
    if unsafe {
        OpenProcessToken(
            proc_handle,
            TOKEN_DUPLICATE | TOKEN_QUERY | TOKEN_ASSIGN_PRIMARY,
            &mut token,
        )
    }
    .is_err()
    {
        tracing::error!("relaunch_unelevated: OpenProcessToken failed");
        return false;
    }
    let _token = OwnedHandle(token);

    let mut lowered = HANDLE::default();
    if unsafe {
        DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut lowered,
        )
    }
    .is_err()
    {
        tracing::error!("relaunch_unelevated: DuplicateTokenEx failed");
        return false;
    }
    let _lowered = OwnedHandle(lowered);

    // Guard: Explorer itself running elevated (rare) would hand us an
    // elevated token and the relaunch would not actually downgrade.
    if is_admin_token(Some(lowered)) {
        tracing::error!("relaunch_unelevated: shell token is elevated");
        return false;
    }

    // Command line: quoted exe path + relaunch marker (single-instance handoff)
    // + passthrough args (`--debug` etc.).
    let args: Vec<String> = std::env::args().skip(1).collect();
    let tail = if args.is_empty() {
        String::new()
    } else {
        format!(" {}", args.join(" "))
    };
    let cmdline = format!(
        "\"{exe}\" {}{tail}",
        crate::platform::single_instance::RELAUNCH_ARG
    );
    let mut cmd = crate::platform::wide_cstr(&cmdline);

    let mut startup = STARTUPINFOW::default();
    startup.cb = size_of::<STARTUPINFOW>() as u32;
    let mut pi = PROCESS_INFORMATION::default();
    let created = unsafe {
        CreateProcessWithTokenW(
            lowered,
            windows::Win32::System::Threading::CREATE_PROCESS_LOGON_FLAGS(0),
            PCWSTR::null(),
            Some(PWSTR(cmd.as_mut_ptr())),
            PROCESS_CREATION_FLAGS(0),
            Some(ptr::null()),
            PCWSTR::null(),
            &startup,
            &mut pi,
        )
    };
    if let Err(e) = created {
        tracing::error!("relaunch_unelevated: CreateProcessWithTokenW failed: {e}");
        return false;
    }
    unsafe {
        let _ = CloseHandle(pi.hProcess);
        let _ = CloseHandle(pi.hThread);
    }
    tracing::info!("unelevated relaunch spawned (pid {})", pi.dwProcessId);
    true
}
