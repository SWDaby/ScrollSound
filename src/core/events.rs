//! Pure event resolution for the low-level mouse hook.
//!
//! Mirrors the C++ `MouseProc` decision logic (Hook.cpp) without any FFI:
//! the wheel sign and the middle-click `dwExtraInfo` guard decide whether a
//! volume event is emitted; taskbar-class matching and the pause flag gate it.

/// A volume action delivered to the system volume controller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeEvent {
    WheelUp,
    WheelDown,
    ToggleMute,
}

/// A raw mouse message as observed by the hook, reduced to pure data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEvent {
    /// `WM_MOUSEWHEEL`; delta is the signed 16-bit wheel rotation (HIWORD of `mouseData`).
    Wheel { delta: i16 },
    /// `WM_MBUTTONDOWN`; `dw_extra_info` guards against double-triggering with
    /// other global hooks (e.g. wgesture).
    MiddleDown { dw_extra_info: usize },
}

/// Decides whether a mouse message over the taskbar yields a volume event.
///
/// C++ parity (Hook.cpp):
/// - wheel delta < 0 (scrolled down) over the taskbar → `WheelDown`;
///   any other delta (including 0) → `WheelUp`;
/// - middle-click over the taskbar with `dw_extra_info == 0` → `ToggleMute`;
/// - anything else, or when paused, → `None`.
pub fn resolve_event(ev: MouseEvent, over_taskbar: bool, paused: bool) -> Option<VolumeEvent> {
    if paused || !over_taskbar {
        return None;
    }
    match ev {
        MouseEvent::Wheel { delta } if delta < 0 => Some(VolumeEvent::WheelDown),
        MouseEvent::Wheel { .. } => Some(VolumeEvent::WheelUp),
        MouseEvent::MiddleDown { dw_extra_info } if dw_extra_info == 0 => {
            Some(VolumeEvent::ToggleMute)
        }
        MouseEvent::MiddleDown { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TASKBAR: bool = true;

    #[test]
    fn negative_delta_over_taskbar_is_wheel_down() {
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: -1 }, TASKBAR, false),
            Some(VolumeEvent::WheelDown)
        );
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: -120 }, TASKBAR, false),
            Some(VolumeEvent::WheelDown)
        );
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: i16::MIN }, TASKBAR, false),
            Some(VolumeEvent::WheelDown)
        );
    }

    #[test]
    fn zero_or_positive_delta_over_taskbar_is_wheel_up() {
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: 0 }, TASKBAR, false),
            Some(VolumeEvent::WheelUp)
        );
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: 120 }, TASKBAR, false),
            Some(VolumeEvent::WheelUp)
        );
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: i16::MAX }, TASKBAR, false),
            Some(VolumeEvent::WheelUp)
        );
    }

    #[test]
    fn wheel_over_non_taskbar_window_is_none() {
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: -120 }, false, false),
            None
        );
        assert_eq!(
            resolve_event(MouseEvent::Wheel { delta: 120 }, false, false),
            None
        );
    }

    #[test]
    fn middle_click_with_nonzero_extra_info_is_none() {
        assert_eq!(
            resolve_event(MouseEvent::MiddleDown { dw_extra_info: 1 }, TASKBAR, false),
            None
        );
        assert_eq!(
            resolve_event(MouseEvent::MiddleDown { dw_extra_info: usize::MAX }, TASKBAR, false),
            None
        );
    }

    #[test]
    fn middle_click_with_zero_extra_info_is_toggle_mute() {
        assert_eq!(
            resolve_event(MouseEvent::MiddleDown { dw_extra_info: 0 }, TASKBAR, false),
            Some(VolumeEvent::ToggleMute)
        );
    }

    #[test]
    fn paused_suppresses_every_event() {
        for ev in [
            MouseEvent::Wheel { delta: -120 },
            MouseEvent::Wheel { delta: 120 },
            MouseEvent::MiddleDown { dw_extra_info: 0 },
        ] {
            assert_eq!(resolve_event(ev, TASKBAR, true), None);
        }
    }
}