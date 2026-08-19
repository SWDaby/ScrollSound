//! Windows version acquisition (design D4): `windows-version` crate,
//! feature-gated mapping to the taskbar class name.

use std::sync::OnceLock;

use windows_version::OsVersion;

use crate::core::table::taskbar_class_for;

static VERSION: OnceLock<OsVersion> = OnceLock::new();

fn current() -> &'static OsVersion {
    VERSION.get_or_init(OsVersion::current)
}

/// The taskbar window class (UTF-16 code units) for the running OS, or `None`
/// when the version is not one of the supported branches.
pub fn taskbar_class() -> Option<Vec<u16>> {
    let v = current();
    taskbar_class_for(v.major, v.minor, v.build)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_is_consistent_with_the_pure_table() {
        // Version acquisition itself is untestable without FFI; assert the
        // wrapper is exactly the pure mapping applied to the current version.
        let v = current();
        assert_eq!(taskbar_class(), taskbar_class_for(v.major, v.minor, v.build));
    }
}