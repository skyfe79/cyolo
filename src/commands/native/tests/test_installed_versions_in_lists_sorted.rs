use super::super::*;
use std::fs;
use tempfile::TempDir;

/// `installed_versions_in` returns every non-dotfile entry, newest-first.
#[test]
fn test_installed_versions_in_lists_sorted() {
    let tmp = TempDir::new().unwrap();
    for name in ["2.1.9", "2.1.10", "2.1.100"] {
        fs::write(tmp.path().join(name), b"binary").unwrap();
    }
    let versions = installed_versions_in(tmp.path()).unwrap();
    assert_eq!(
        versions,
        vec![
            "2.1.100".to_string(),
            "2.1.10".to_string(),
            "2.1.9".to_string()
        ]
    );
}
