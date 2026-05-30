use super::super::*;
use std::fs;
use tempfile::TempDir;

/// Dotfile entries (`.DS_Store`, a leftover `.claude.cyolo-tmp` link, etc.)
/// must never be reported as installed versions.
#[test]
fn test_installed_versions_in_skips_dotfiles() {
    let tmp = TempDir::new().unwrap();
    fs::write(tmp.path().join("2.1.5"), b"binary").unwrap();
    fs::write(tmp.path().join(".DS_Store"), b"junk").unwrap();
    fs::write(tmp.path().join(".claude.cyolo-tmp"), b"junk").unwrap();

    let versions = installed_versions_in(tmp.path()).unwrap();
    assert_eq!(versions, vec!["2.1.5".to_string()]);
}
