use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_gitdir_resolves_worktree_file_with_relative_gitdir() {
    let tmp = TempDir::new().unwrap();
    let worktree = tmp.path().join("worktree-checkout");
    fs::create_dir(&worktree).unwrap();
    // Relative path is interpreted against the .git file's parent (= worktree).
    fs::create_dir_all(worktree.join("nested").join("gitdir")).unwrap();
    fs::write(worktree.join(".git"), "gitdir: nested/gitdir\n").unwrap();

    let found = find_gitdir(&worktree).unwrap();
    assert_eq!(found, worktree.join("nested").join("gitdir"));
}
