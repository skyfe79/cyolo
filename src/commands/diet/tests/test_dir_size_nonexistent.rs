use super::super::*;
use super::common::*;

#[test]
fn test_dir_size_nonexistent() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent");
    assert_eq!(dir_size(&path), 0);
}
