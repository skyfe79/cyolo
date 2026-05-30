use super::super::*;
use crate::error::CyoloError;
use std::fs;
use tempfile::TempDir;

/// Switching to an absent version errors with VersionNotInstalled and leaves
/// the existing launcher untouched (never a dangling link).
#[test]
fn test_switch_in_missing_version_errors() {
    let tmp = TempDir::new().unwrap();
    let versions = tmp.path().join("versions");
    fs::create_dir(&versions).unwrap();
    let v1 = versions.join("2.1.154");
    fs::write(&v1, b"old").unwrap();

    let bin = tmp.path().join("claude");
    std::os::unix::fs::symlink(&v1, &bin).unwrap();

    match switch_in(&bin, &versions, "9.9.9") {
        Err(CyoloError::VersionNotInstalled { version }) => assert_eq!(version, "9.9.9"),
        other => panic!("expected VersionNotInstalled, got {other:?}"),
    }
    // Launcher still points at the original version.
    assert_eq!(fs::read_link(&bin).unwrap(), v1);
}
