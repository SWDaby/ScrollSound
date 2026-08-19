//! Volume control boundary (design D11): the one trait where platforms differ.
//!
//! The Win32 implementation sends the system media keys (`VK_VOLUME_*` via
//! `SendInput`) — the exact input path of the keyboard volume keys: native
//! one-step volume changes, native volume flyout, native mute toggle.
//!
//! History (2026-08): the C++ `PostMessage(WM_APPCOMMAND)` transport is
//! ignored by modern shells (verified on build 26200); direct Core Audio
//! (`IAudioEndpointVolume`, hand-vtabled) changes the volume but bypasses the
//! shell, so no volume flyout shows. Media keys give the original
//! keyboard-shortcut UX with zero FFI surface beyond the crate wrapper.

use std::mem::size_of;

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT_KEYBOARD, INPUT_0, INPUT, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP,
    VK_VOLUME_DOWN, VK_VOLUME_MUTE, VK_VOLUME_UP, VIRTUAL_KEY,
};

use crate::core::events::VolumeEvent;

/// Delivers volume events to the system. Implementations must be callable from
/// the hook callback (no blocking).
pub trait VolumeController {
    /// Applies a wheel event (`WheelUp`/`WheelDown`); `ToggleMute` is
    /// deliberately ignored — mute has its own method.
    fn wheel(&self, dir: VolumeEvent);
    fn toggle_mute(&self);
}

/// Media-key implementation.
#[derive(Clone, Copy)]
pub struct WinVolumeController;

impl WinVolumeController {
    pub const fn new() -> Self {
        Self
    }
}

impl VolumeController for WinVolumeController {
    fn wheel(&self, dir: VolumeEvent) {
        let key = match dir {
            VolumeEvent::WheelUp => VK_VOLUME_UP,
            VolumeEvent::WheelDown => VK_VOLUME_DOWN,
            VolumeEvent::ToggleMute => return,
        };
        press_media_key(key);
    }

    fn toggle_mute(&self) {
        press_media_key(VK_VOLUME_MUTE);
    }
}

/// Sends a full key press (down + up) of a media key.
fn press_media_key(key: VIRTUAL_KEY) {
    let down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    let sent = unsafe { SendInput(&[down, up], size_of::<INPUT>() as i32) };
    tracing::trace!(key = format_args!("{key:?}"), sent, "media key");
}