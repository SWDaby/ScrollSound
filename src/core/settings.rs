//! `setting.ini` admin-flag parsing (pure).
//!
//! Windows profile APIs already implement the parse; this is the decision
//! function over their result, kept pure so the boundary is testable.

/// Admin mode is enabled iff the raw profile value is exactly 1
/// (spec: `elevation` capability — `1` = enabled, anything else = disabled,
/// including a missing file, which the profile API reports as the default).
pub fn admin_flag(raw: i32) -> bool {
    raw == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_enabled_only_for_value_one() {
        assert!(admin_flag(1));
        assert!(!admin_flag(0));
        assert!(!admin_flag(2));
        assert!(!admin_flag(-1));
        assert!(!admin_flag(i32::MIN));
        assert!(!admin_flag(i32::MAX));
    }
}