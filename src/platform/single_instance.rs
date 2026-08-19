//! Single-instance guard via a session-scoped named mutex.
//!
//! `acquire_now` is the common path: a duplicate launch exits immediately.
//! The elevated/unelevated relaunch child (`--elevated-relaunch`) overlaps
//! its parent for a moment (the parent exits right after spawning it), so it
//! uses `acquire_wait`: poll until the parent releases the mutex. A crashed
//! previous instance leaves the mutex unowned, so the next launch acquires it
//! normally — no stale-lock deadlock.

use std::time::{Duration, Instant};

use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{CreateMutexW, WaitForSingleObject};

use crate::platform::wide_cstr;

/// Session-scoped: one instance per logon session. `Local\` makes the mutex
/// visible to both an unelevated and an elevated instance of the same user.
const MUTEX_NAME: &str = r"Local\ScrollSound.SingleInstance";

/// Argument appended by both relaunch directions; tells the child to wait for
/// the mutex handoff instead of exiting as a duplicate.
pub const RELAUNCH_ARG: &str = "--elevated-relaunch";

/// Holds the mutex for the process lifetime; drop releases it.
pub struct InstanceGuard(HANDLE);

impl Drop for InstanceGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Acquires the single-instance lock immediately. `None` = another instance
/// currently owns it.
pub fn acquire_now() -> Option<InstanceGuard> {
    let name = wide_cstr(MUTEX_NAME);
    let handle = unsafe { CreateMutexW(None, true, PCWSTR(name.as_ptr())) }.ok()?;
    // We asked for initial ownership: a non-zero wait means another process
    // currently owns the mutex, so this launch is a duplicate. (WAIT_FAILED
    // also lands here — fail closed, never run two instances.)
    if unsafe { WaitForSingleObject(handle, 0) } == WAIT_OBJECT_0 {
        Some(InstanceGuard(handle))
    } else {
        unsafe {
            let _ = CloseHandle(handle);
        }
        None
    }
}

/// Acquires the lock, polling until `timeout`, for relaunch-handoff children
/// whose parent process is still exiting. `None` = still held after the
/// deadline.
pub fn acquire_wait(timeout: Duration) -> Option<InstanceGuard> {
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(guard) = acquire_now() {
            return Some(guard);
        }
        if Instant::now() >= deadline {
            return None;
        }
        std::thread::sleep(Duration::from_millis(150));
    }
}
