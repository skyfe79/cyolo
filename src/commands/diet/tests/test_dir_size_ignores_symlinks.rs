use super::super::*;
use super::common::*;

#[test]
fn test_dir_size_ignores_symlinks() {
    let dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create a real file in the target (should NOT be counted)
    fs::write(target_dir.path().join("big.bin"), vec![0u8; 1000]).unwrap();

    // Create a symlink inside the scanned dir pointing to the target
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target_dir.path(), dir.path().join("link")).unwrap();
    }

    // Create a real file alongside the symlink
    fs::write(dir.path().join("real.txt"), "data").unwrap(); // 4 bytes

    // dir_size should only count the real file, NOT follow the symlink
    assert_eq!(dir_size(dir.path()), 4);
}
