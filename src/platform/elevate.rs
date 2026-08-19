//! Elevation: admin-token detection and `runas` relaunch (design D7, spec:
//! `elevation`). Runs before any UI; the launch gate lives in `main.rs`.

use std::mem::size_of;

use windows::core::{BOOL, PCWSTR};
use windows::Win32::Security::{
    AllocateAndInitializeSid, CheckTokenMembership, FreeSid, PSID, SID_IDENTIFIER_AUTHORITY,
};
use windows::Win32::UI::Shell::{ShellExecuteExW, SHELLEXECUTEINFOW};

use crate::platform::win_settings;

// Values from winnt.h/winbase.h, kept local (plain constants).
const SECURITY_NT_AUTHORITY: [u8; 6] = [0, 0, 0, 0, 0, 5];
const SECURITY_BUILTIN_DOMAIN_RID: u32 = 0x20;
const DOMAIN_ALIAS_RID_ADMINS: u32 = 0x220;

/// Whether the current process token is a member of the Administrators group.
pub fn is_admin() -> bool {
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
            0, 0, 0, 0, 0, 0,
            &mut sid,
        )
    };
    if allocated.is_err() {
        return false;
    }
    let mut member = BOOL::default();
    let checked = unsafe { CheckTokenMembership(None, sid, &mut member) };
    unsafe {
        FreeSid(sid);
    }
    checked.is_ok() && member.as_bool()
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
    let params = if args.is_empty() {
        Vec::new()
    } else {
        crate::platform::wide_cstr(&args.join(" "))
    };
    let mut sei = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        ..Default::default()
    };
    unsafe { ShellExecuteExW(&mut sei) }.is_ok()
}