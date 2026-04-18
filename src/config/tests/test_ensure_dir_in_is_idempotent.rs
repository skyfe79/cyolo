use super::super::*;
use tempfile::TempDir;

#[test]
fn test_ensure_dir_in_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    ensure_dir_in(tmp.path()).unwrap();
    assert!(config_dir_in(tmp.path()).is_dir());
    // Second call must not error when the directory already exists.
    ensure_dir_in(tmp.path()).unwrap();
    assert!(config_dir_in(tmp.path()).is_dir());
}
