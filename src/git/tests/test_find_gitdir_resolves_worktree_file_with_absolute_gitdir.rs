use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_gitdir_resolves_worktree_file_with_absolute_gitdir() {
    let tmp = TempDir::new().unwrap();
    let real_gitdir = tmp.path().join("shared-repo").join("worktrees").join("feature");
    fs::create_dir_all(&real_gitdir).unwrap();
    let worktree = tmp.path().join("worktree-checkout");
    fs::create_dir(&worktree).unwrap();
    fs::write(
        worktree.join(".git"),
        format!("gitdir: {}\n", real_gitdir.display()),
    )
    .unwrap();

    let found = find_gitdir(&worktree).unwrap();
    assert_eq!(found, real_gitdir);
}
