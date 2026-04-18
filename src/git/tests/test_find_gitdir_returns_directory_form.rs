use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_gitdir_returns_directory_form() {
    let tmp = TempDir::new().unwrap();
    let git = tmp.path().join(".git");
    fs::create_dir(&git).unwrap();

    let found = find_gitdir(tmp.path()).unwrap();
    assert_eq!(found, git);
}
