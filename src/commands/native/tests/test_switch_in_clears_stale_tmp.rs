use super::super::*;
use std::fs;
use tempfile::TempDir;

/// A leftover temp symlink from a crashed prior run must not block the swap —
/// `switch_in` clears `.<name>.cyolo-tmp` before creating its own.
#[test]
fn test_switch_in_clears_stale_tmp() {
    let tmp = TempDir::new().unwrap();
    let versions = tmp.path().join("versions");
    fs::create_dir(&versions).unwrap();
    let v1 = versions.join("2.1.154");
    let v2 = versions.join("2.1.156");
    fs::write(&v1, b"old").unwrap();
    fs::write(&v2, b"new").unwrap();

    let bin = tmp.path().join("claude");
    std::os::unix::fs::symlink(&v1, &bin).unwrap();

    // Simulate a stale temp link left behind by an interrupted earlier switch.
    let stale = tmp.path().join(".claude.cyolo-tmp");
    std::os::unix::fs::symlink(&v1, &stale).unwrap();

    switch_in(&bin, &versions, "2.1.156").unwrap();

    assert_eq!(fs::read_link(&bin).unwrap(), v2);
    assert!(!stale.exists(), "stale temp link should be gone");
}
