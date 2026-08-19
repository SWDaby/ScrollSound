//! Global low-level mouse hook (`WH_MOUSE_LL`) and its callback.
//!
//! Hot-path constraints (design D2/D11): the callback only reads the
//! `OnceLock`-filled class and the atomic pause flag, and sends volume media
//! keys — no locks, no allocation, no panics. State is shared with the menu
//! handlers in `taskbar_menu.rs`/`main.rs` via the statics below.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, GetClassNameW, GetCursorPos, SetWindowsHookExW, UnhookWindowsHookEx,
    WindowFromPoint, MSLLHOOKSTRUCT, HHOOK, WH_MOUSE_LL, WM_MBUTTONDOWN, WM_MOUSEWHEEL,
};

use crate::core::events::{resolve_event, MouseEvent};
use crate::platform::volume::{VolumeController, WinVolumeController};
use crate::platform::win_version;

/// Taskbar window class for the running OS, filled once at startup.
pub static TASKBAR_CLASS: OnceLock<Vec<u16>> = OnceLock::new();

/// Pause flag (C++ `isPause`): `true` = hook active. Toggled by the 暂停/继续
/// menu item and the re-hook action; the watchdog re-hooks only when active.
pub static RUNNING: AtomicBool = AtomicBool::new(true);

/// Handle of the currently installed hook, if any (stored as the raw `HHOOK`
/// address so the static stays `Send`). Only ever touched on the pump (main)
/// thread — the callback never takes this lock.
static HOOK: Mutex<Option<isize>> = Mutex::new(None);

/// Fills `TASKBAR_CLASS` from the running OS version (design D2: version/class
/// init happens at startup, before the hook is installed).
pub fn init_taskbar_class() {
    if let Some(class) = win_version::taskbar_class() {
        let _ = TASKBAR_CLASS.set(class);
    }
}

/// Installs the global low-level mouse hook on the given thread (0 = all).
/// Returns whether the install succeeded.
pub fn set_mouse_hook(thread_id: u32) -> bool {
    let hook = unsafe { SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_proc), None, thread_id) };
    match hook {
        Ok(h) => {
            if let Ok(mut slot) = HOOK.lock() {
                *slot = Some(h.0 as isize);
            }
            tracing::info!("mouse hook installed");
            true
        }
        Err(e) => {
            tracing::error!("SetWindowsHookExW failed: {e}");
            false
        }
    }
}

/// Uninstalls the hook (C++ `MoveMouseHook`).
pub fn move_mouse_hook() {
    let removed = HOOK.lock().map(|mut slot| slot.take()).unwrap_or(None);
    if let Some(addr) = removed {
        let h = HHOOK(addr as *mut core::ffi::c_void);
        unsafe {
            let _ = UnhookWindowsHookEx(h);
        }
        tracing::info!("mouse hook removed");
    }
}

/// Forced re-installation (C++ `ReHook`): keeps the hook near the head of the
/// hook chain; also used by the 重新HOOK menu item and the watchdog. Returns
/// whether the final install succeeded.
pub fn re_hook() -> bool {
    move_mouse_hook();
    set_mouse_hook(0)
}

unsafe extern "system" fn mouse_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
        let raw_msg = wparam.0 as u32;
        let mouse_ev = match raw_msg {
            WM_MOUSEWHEEL => Some(MouseEvent::Wheel {
                delta: (info.mouseData >> 16) as u16 as i16,
            }),
            WM_MBUTTONDOWN => Some(MouseEvent::MiddleDown {
                dw_extra_info: info.dwExtraInfo,
            }),
            _ => None,
        };
        if let Some(ev) = mouse_ev {
            // C++ parity: `GetCursorPos` (this process's coordinate space),
            // not `MSLLHOOKSTRUCT.pt` (physical pixels). Since the process is
            // DPI-unaware, mixing the two spaces breaks WindowFromPoint on
            // scaled displays.
            let mut pt = POINT::default();
            let _ = unsafe { GetCursorPos(&mut pt) };
            let hwnd = WindowFromPoint(pt);
            let over_taskbar = class_matches(hwnd, TASKBAR_CLASS.get());
            let paused = !RUNNING.load(Ordering::Relaxed);
            let observed = observed_class(hwnd);
            tracing::trace!(msg = format_args!("0x{raw_msg:04X}"), event = ?ev, over_taskbar, %observed, x = pt.x, y = pt.y, "hook event");
            if let Some(action) = resolve_event(ev, over_taskbar, paused) {
                let ctl = WinVolumeController::new();
                match action {
                    crate::core::events::VolumeEvent::ToggleMute => ctl.toggle_mute(),
                    dir => ctl.wheel(dir),
                }
            }
        }
    }
    // Always forward down the hook chain.
    CallNextHookEx(None, code, wparam, lparam)
}

/// Whether the window's class equals the expected taskbar class.
fn class_matches(hwnd: HWND, expected: Option<&Vec<u16>>) -> bool {
    let Some(expected) = expected else {
        return false;
    };
    if hwnd.0.is_null() {
        return false;
    }
    let mut buf = [0u16; 128];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    if len <= 0 {
        return false;
    }
    let len = len as usize;
    len == expected.len() && buf[..len] == expected[..]
}

/// Debug aid (trace-level only): the class of the window under the cursor.
fn observed_class(hwnd: HWND) -> String {
    if hwnd.0.is_null() {
        return "<null hwnd>".to_string();
    }
    let mut buf = [0u16; 128];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) };
    if len <= 0 {
        return format!("<class read failed: {len}>");
    }
    String::from_utf16_lossy(&buf[..len as usize])
}