use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ensure_exclude_entry_appends_after_missing_trailing_newline() {
    let tmp = TempDir::new().unwrap();
    let gitdir = tmp.path().to_path_buf();
    fs::create_dir_all(gitdir.join("info")).unwrap();
    // Pre-existing file with NO trailing newline — a common hand-edited shape.
    fs::write(gitdir.join("info").join("exclude"), "*.log").unwrap();

    assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

    let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
    assert_eq!(contents, "*.log\n.claude-profile.json\n");
}
