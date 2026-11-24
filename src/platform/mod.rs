//! Platform layer — the single `#[cfg]` re-export point (design D10/D11).
//! A future non-Windows port swaps exactly this surface.

/// Wide (UTF-16) string with a trailing NUL, for Win32 string parameters.
pub(crate) fn wide_cstr(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(windows)]
pub mod elevate;
#[cfg(windows)]
pub mod hook;
#[cfg(windows)]
pub mod single_instance;
#[cfg(windows)]
pub mod volume;
#[cfg(windows)]
pub mod win_autostart;
#[cfg(windows)]
pub mod win_settings;
#[cfg(windows)]
pub mod win_version;
