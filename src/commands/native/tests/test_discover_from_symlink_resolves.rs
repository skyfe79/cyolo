use super::super::*;
use std::fs;
use tempfile::TempDir;

/// A launcher symlinked into `versions/<ver>` resolves to the right
/// versions_dir + current version — the real native-install shape.
#[test]
fn test_discover_from_symlink_resolves() {
    let tmp = TempDir::new().unwrap();
    let versions = tmp.path().join("versions");
    fs::create_dir(&versions).unwrap();
    let target = versions.join("2.1.156");
    fs::write(&target, b"binary").unwrap();

    let bin = tmp.path().join("claude");
    std::os::unix::fs::symlink(&target, &bin).unwrap();

    let install = discover_from(&bin).unwrap();
    assert_eq!(install.bin_link, bin);
    assert_eq!(install.versions_dir, versions);
    assert_eq!(install.current, Some("2.1.156".to_string()));
}
