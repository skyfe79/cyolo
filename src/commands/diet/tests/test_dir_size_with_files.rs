use super::super::*;
use super::common::*;

#[test]
fn test_dir_size_with_files() {
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join("a.txt"), "hello").unwrap(); // 5 bytes
    fs::write(dir.path().join("b.txt"), "world!").unwrap(); // 6 bytes
    assert_eq!(dir_size(dir.path()), 11);
}
