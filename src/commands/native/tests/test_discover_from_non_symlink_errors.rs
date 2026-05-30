use super::super::*;
use crate::error::CyoloError;
use std::fs;
use tempfile::TempDir;

/// A plain (non-symlink) launcher — e.g. an npm/global install or a copied
/// binary — can't be repointed, so discovery rejects it with NotNativeInstall.
#[test]
fn test_discover_from_non_symlink_errors() {
    let tmp = TempDir::new().unwrap();
    let bin = tmp.path().join("claude");
    fs::write(&bin, b"binary").unwrap();

    match discover_from(&bin) {
        Err(CyoloError::NotNativeInstall { path }) => assert_eq!(path, bin),
        other => panic!("expected NotNativeInstall, got {other:?}"),
    }
}
