use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ensure_exclude_entry_is_idempotent() {
    let tmp = TempDir::new().unwrap();
    let gitdir = tmp.path().to_path_buf();

    assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());
    // Second call must not duplicate.
    assert!(!ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

    let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
    assert_eq!(contents.matches(".claude-profile.json").count(), 1);
}
