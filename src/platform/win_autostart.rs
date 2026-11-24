//! Autostart via the `Run` registry value (spec: `autostart`; design D5 area).
//!
//! Value name `ScrollSound V1.24` under `HKCU\...\CurrentVersion\Run` must
//! stay identical to the C++ (`APP_NAME`) or existing entries are orphaned.
//! Registry errors propagate as `io::Result` — never panics.

use std::ffi::c_void;
use std::io;

use windows::core::PCWSTR;
use windows::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS, WIN32_ERROR,
};
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegGetValueW, RegOpenKeyExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE, REG_SAM_FLAGS,
    REG_SZ, RRF_RT_REG_SZ,
};

use crate::core::autostart;
use crate::platform::wide_cstr;

const RUN_KEY: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
const VALUE_NAME: &str = "ScrollSound V1.24";

fn err_from(win32: WIN32_ERROR, what: &str) -> io::Error {
    io::Error::other(format!("{what}: Win32 error {}", win32.0))
}

fn open_key(sam: REG_SAM_FLAGS) -> io::Result<HKEY> {
    let run_key = wide_cstr(RUN_KEY);
    let mut key = HKEY::default();
    let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(run_key.as_ptr()), None, sam, &mut key) };
    if rc == ERROR_SUCCESS {
        Ok(key)
    } else {
        Err(err_from(rc, "RegOpenKeyExW(Run)"))
    }
}

/// Reads the current value of `ScrollSound V1.24`; `None` when absent.
fn read_value() -> io::Result<Option<String>> {
    let key = open_key(KEY_READ | KEY_QUERY_VALUE)?;
    let result = (|| {
        let name = wide_cstr(VALUE_NAME);
        // First call with no buffer to learn the required size.
        let mut size: u32 = 0;
        let rc = unsafe {
            RegGetValueW(
                key,
                None,
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            )
        };
        if rc == ERROR_SUCCESS && size == 0 {
            return Ok(None); // empty value
        }
        if rc == ERROR_FILE_NOT_FOUND {
            return Ok(None); // value absent → autostart disabled
        }
        if rc != ERROR_MORE_DATA && rc != ERROR_SUCCESS {
            return Err(err_from(rc, "RegGetValueW(size)"));
        }
        let count = (size as usize + 1) / 2; // room for the NUL
        let mut buf = vec![0u16; count];
        let mut got: u32 = size;
        let rc = unsafe {
            RegGetValueW(
                key,
                None,
                PCWSTR(name.as_ptr()),
                RRF_RT_REG_SZ,
                None,
                Some(buf.as_mut_ptr() as *mut c_void),
                Some(&mut got),
            )
        };
        if rc != ERROR_SUCCESS {
            return Err(err_from(rc, "RegGetValueW(value)"));
        }
        let len = (got as usize).min(buf.len());
        buf.truncate(len);
        while buf.last() == Some(&0) {
            buf.pop();
        }
        Ok(Some(String::from_utf16_lossy(&buf)))
    })();
    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

/// Autostart is enabled iff the value exists and equals the exe path.
pub fn is_enabled(exe_path: &str) -> io::Result<bool> {
    Ok(read_value()?.is_some_and(|v| autostart::is_enabled(Some(&v), exe_path)))
}

/// Writes the exe path to the value (no-op when already enabled).
pub fn enable(exe_path: &str) -> io::Result<()> {
    let key = open_key(KEY_SET_VALUE | KEY_QUERY_VALUE)?;
    let result = (|| {
        if is_enabled(exe_path)? {
            return Ok(());
        }
        let name = wide_cstr(VALUE_NAME);
        // REG_SZ data must be NUL-terminated (matches the C++ `+1` length).
        let mut data = wide_cstr(exe_path);
        data.push(0);
        let bytes: &[u8] =
            unsafe { std::slice::from_raw_parts(data.as_ptr() as *const u8, data.len() * 2) };
        let rc = unsafe { RegSetValueExW(key, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes)) };
        if rc == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(err_from(rc, "RegSetValueExW(Run)"))
        }
    })();
    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}

/// Deletes the value (no-op when already disabled).
pub fn disable() -> io::Result<()> {
    let key = open_key(KEY_SET_VALUE | KEY_QUERY_VALUE)?;
    let result = (|| {
        let name = wide_cstr(VALUE_NAME);
        let rc = unsafe { RegDeleteValueW(key, PCWSTR(name.as_ptr())) };
        if rc == ERROR_SUCCESS {
            Ok(())
        } else {
            Err(err_from(rc, "RegDeleteValueW(Run)"))
        }
    })();
    unsafe {
        let _ = RegCloseKey(key);
    }
    result
}