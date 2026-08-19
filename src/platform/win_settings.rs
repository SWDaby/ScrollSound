//! `setting.ini` admin flag via Win32 profile APIs (design D6).
//!
//! Same file format and semantics as the C++ (`[setting] Administrator=1`),
//! with the documented improvement: the file is resolved against the
//! executable's directory instead of the CWD.

use std::io;
use std::path::PathBuf;

use windows::core::PCWSTR;
use windows::Win32::Foundation::MAX_PATH;
use windows::Win32::Storage::FileSystem::GetFullPathNameW;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::WindowsProgramming::{
    GetPrivateProfileIntW, WritePrivateProfileStringW,
};

use crate::core::settings::admin_flag;
use crate::platform::wide_cstr;

const SECTION: &str = "setting";
const KEY: &str = "Administrator";

/// Full path of the current executable, normalized via `GetFullPathNameW`
/// (design D6: normalize module path before deriving the directory).
pub fn exe_path() -> Option<String> {
    let mut buf = [0u16; MAX_PATH as usize];
    let len = unsafe { GetModuleFileNameW(None, &mut buf) };
    if len == 0 {
        return None;
    }
    let raw = &buf[..len as usize];
    let mut out = [0u16; MAX_PATH as usize];
    let n = unsafe { GetFullPathNameW(PCWSTR(raw.as_ptr()), Some(&mut out), None) };
    if n == 0 {
        return None;
    }
    let end = (n as usize).min(out.len());
    Some(String::from_utf16_lossy(&out[..end]))
}

/// Directory containing the executable.
pub fn exe_dir() -> Option<PathBuf> {
    exe_path().and_then(|p| PathBuf::from(p).parent().map(|d| d.to_path_buf()))
}

/// `setting.ini` next to the executable.
pub fn setting_ini_path() -> Option<PathBuf> {
    exe_dir().map(|d| d.join("setting.ini"))
}

/// Adopts a legacy C++ `setting.ini` once: when the exe-dir file is absent but
/// a CWD-relative one exists (the C++ resolved `.\setting.ini` against its
/// working directory), copy it to the exe dir before the state is read.
/// After that the exe dir is the single source of truth.
pub fn migrate_legacy_setting() {
    let Some(dir) = exe_dir() else {
        return;
    };
    let target = dir.join("setting.ini");
    if target.exists() {
        return;
    }
    let Ok(legacy) = std::env::current_dir().map(|cwd| cwd.join("setting.ini")) else {
        return;
    };
    if legacy.exists() {
        match std::fs::copy(&legacy, &target) {
            Ok(_) => tracing::info!("adopted legacy setting.ini from {:?}", legacy),
            Err(e) => tracing::warn!("cannot adopt legacy setting.ini: {e}"),
        }
    }
}

/// Admin mode is enabled iff `[setting] Administrator=1` (absent file → 0).
pub fn admin_enabled() -> bool {
    let Some(path) = setting_ini_path() else {
        return false;
    };
    let app = wide_cstr(SECTION);
    let key = wide_cstr(KEY);
    let file = wide_cstr(&path.to_string_lossy());
    let raw = unsafe {
        GetPrivateProfileIntW(
            PCWSTR(app.as_ptr()),
            PCWSTR(key.as_ptr()),
            0,
            PCWSTR(file.as_ptr()),
        )
    };
    admin_flag(raw)
}

/// Writes `Administrator=1` / `Administrator=0`, preserving other content.
pub fn set_admin(enabled: bool) -> io::Result<()> {
    let Some(path) = setting_ini_path() else {
        return Err(io::Error::other("cannot resolve executable directory"));
    };
    let app = wide_cstr(SECTION);
    let key = wide_cstr(KEY);
    let value = if enabled { "1" } else { "0" };
    let value = wide_cstr(value);
    let file = wide_cstr(&path.to_string_lossy());
    let rc = unsafe {
        WritePrivateProfileStringW(
            PCWSTR(app.as_ptr()),
            PCWSTR(key.as_ptr()),
            PCWSTR(value.as_ptr()),
            PCWSTR(file.as_ptr()),
        )
    };
    rc.map_err(|e| io::Error::other(format!("WritePrivateProfileStringW: {e}")))
}