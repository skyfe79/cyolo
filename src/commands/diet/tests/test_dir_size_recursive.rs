use super::super::*;
use super::common::*;

#[test]
fn test_dir_size_recursive() {
    let dir = TempDir::new().unwrap();
    fs::create_dir(dir.path().join("sub")).unwrap();
    fs::write(dir.path().join("a.txt"), "hi").unwrap(); // 2 bytes
    fs::write(dir.path().join("sub").join("b.txt"), "hey").unwrap(); // 3 bytes
    assert_eq!(dir_size(dir.path()), 5);
}
