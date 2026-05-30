use super::super::*;
use std::fs;
use tempfile::TempDir;

/// Switching repoints the launcher symlink at the requested version's binary,
/// atomically replacing the previous target.
#[test]
fn test_switch_in_repoints_symlink() {
    let tmp = TempDir::new().unwrap();
    let versions = tmp.path().join("versions");
    fs::create_dir(&versions).unwrap();
    let v1 = versions.join("2.1.154");
    let v2 = versions.join("2.1.156");
    fs::write(&v1, b"old").unwrap();
    fs::write(&v2, b"new").unwrap();

    let bin = tmp.path().join("claude");
    std::os::unix::fs::symlink(&v1, &bin).unwrap();

    switch_in(&bin, &versions, "2.1.156").unwrap();

    assert!(bin.is_symlink());
    assert_eq!(fs::read_link(&bin).unwrap(), v2);
}
