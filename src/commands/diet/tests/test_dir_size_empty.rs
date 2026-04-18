use super::super::*;
use super::common::*;

#[test]
fn test_dir_size_empty() {
    let dir = TempDir::new().unwrap();
    assert_eq!(dir_size(dir.path()), 0);
}
