//! Taskbar window-class table: version → class name.
//!
//! Pure mapping of the three `Hook.cpp` branches. The class names are returned
//! as UTF-16 code units (what `GetClassNameW` compares against) without a NUL
//! terminator.

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Maps an OS version (major/minor/build) to the taskbar window class that
/// owns the empty taskbar area, per `Hook.cpp`:
///
/// - Windows 10 (10.0, build ≤ 21390) → `MSTaskListWClass`
/// - Windows 11 21H2 (10.0, build 22000) → `ReBarWindow32`
/// - Windows 11 22H2+ (10.0, build ≥ 22621) → `Shell_TrayWnd`
/// - anything else → `None` (hook installs, no class matched, feature inert)
pub fn taskbar_class_for(major: u32, minor: u32, build: u32) -> Option<Vec<u16>> {
    if major != 10 || minor != 0 {
        return None;
    }
    if build <= 21390 {
        Some(wide("MSTaskListWClass"))
    } else if build == 22000 {
        Some(wide("ReBarWindow32"))
    } else if build >= 22621 {
        Some(wide("Shell_TrayWnd"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn class(v: &[u16]) -> String {
        String::from_utf16_lossy(v)
    }

    #[test]
    fn windows10_uses_classic_taskbar_class() {
        assert_eq!(
            class(&taskbar_class_for(10, 0, 19045).unwrap()),
            "MSTaskListWClass"
        );
        assert_eq!(
            class(&taskbar_class_for(10, 0, 21390).unwrap()),
            "MSTaskListWClass"
        );
    }

    #[test]
    fn windows11_21h2_uses_rebar_class() {
        assert_eq!(
            class(&taskbar_class_for(10, 0, 22000).unwrap()),
            "ReBarWindow32"
        );
    }

    #[test]
    fn windows11_22h2_or_later_uses_shell_tray_class() {
        assert_eq!(
            class(&taskbar_class_for(10, 0, 22621).unwrap()),
            "Shell_TrayWnd"
        );
        assert_eq!(
            class(&taskbar_class_for(10, 0, 26200).unwrap()),
            "Shell_TrayWnd"
        );
    }

    #[test]
    fn unknown_versions_yield_none() {
        assert_eq!(taskbar_class_for(6, 1, 7601), None);
        assert_eq!(taskbar_class_for(10, 0, 22001), None);
        assert_eq!(taskbar_class_for(10, 0, 21391), None);
        assert_eq!(taskbar_class_for(11, 0, 22621), None);
    }
}