use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_gitdir_walks_up_from_subdirectory() {
    let tmp = TempDir::new().unwrap();
    let git = tmp.path().join(".git");
    fs::create_dir(&git).unwrap();
    let deep = tmp.path().join("a").join("b").join("c");
    fs::create_dir_all(&deep).unwrap();

    let found = find_gitdir(&deep).unwrap();
    assert_eq!(found, git);
}
