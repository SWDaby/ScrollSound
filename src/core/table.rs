//! Taskbar window-class table: version → class name.
//!
//! Pure mapping of the three `Hook.cpp` branches. The class names are returned
//! as UTF-16 code units (what `GetClassNameW` compares against) without a NUL
//! terminator.

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().collect()
}

/// Maps an OS version (major/minor/build) to the set of taskbar window
/// classes that own the empty taskbar area, per `Hook.cpp`:
///
/// - Windows 10 (10.0, build ≤ 21390) → `MSTaskListWClass` (one class per
///   monitor taskbar, so a single entry covers multi-monitor)
/// - Windows 11 21H2 (10.0, build 22000) → `ReBarWindow32`
/// - Windows 11 22H2+ (10.0, build ≥ 22621) → `Shell_TrayWnd` (primary
///   monitor) + `Shell_SecondaryTrayWnd` (secondary-monitor taskbars, which
///   are separate top-level windows on Win11)
/// - anything else → `None` (hook installs, no class matched, feature inert)
pub fn taskbar_class_for(major: u32, minor: u32, build: u32) -> Option<Vec<Vec<u16>>> {
    if major != 10 || minor != 0 {
        return None;
    }
    if build <= 21390 {
        Some(vec![wide("MSTaskListWClass")])
    } else if build == 22000 {
        Some(vec![wide("ReBarWindow32")])
    } else if build >= 22621 {
        Some(vec![wide("Shell_TrayWnd"), wide("Shell_SecondaryTrayWnd")])
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn classes(v: &[Vec<u16>]) -> Vec<String> {
        v.iter().map(|c| String::from_utf16_lossy(c)).collect()
    }

    #[test]
    fn windows10_uses_classic_taskbar_class() {
        assert_eq!(
            classes(&taskbar_class_for(10, 0, 19045).unwrap()),
            ["MSTaskListWClass"]
        );
        assert_eq!(
            classes(&taskbar_class_for(10, 0, 21390).unwrap()),
            ["MSTaskListWClass"]
        );
    }

    #[test]
    fn windows11_21h2_uses_rebar_class() {
        assert_eq!(
            classes(&taskbar_class_for(10, 0, 22000).unwrap()),
            ["ReBarWindow32"]
        );
    }

    #[test]
    fn windows11_22h2_or_later_matches_primary_and_secondary_taskbars() {
        let expected = ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"];
        assert_eq!(classes(&taskbar_class_for(10, 0, 22621).unwrap()), expected);
        assert_eq!(classes(&taskbar_class_for(10, 0, 26200).unwrap()), expected);
    }

    #[test]
    fn unknown_versions_yield_none() {
        assert_eq!(taskbar_class_for(6, 1, 7601), None);
        assert_eq!(taskbar_class_for(10, 0, 22001), None);
        assert_eq!(taskbar_class_for(10, 0, 21391), None);
        assert_eq!(taskbar_class_for(11, 0, 22621), None);
    }
}
