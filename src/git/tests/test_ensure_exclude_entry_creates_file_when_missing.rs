use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ensure_exclude_entry_creates_file_when_missing() {
    let tmp = TempDir::new().unwrap();
    let gitdir = tmp.path().to_path_buf();

    let added = ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap();
    assert!(added);

    let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
    assert_eq!(contents, ".claude-profile.json\n");
}
