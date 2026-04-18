use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ensure_exclude_entry_detects_existing_with_whitespace() {
    let tmp = TempDir::new().unwrap();
    let gitdir = tmp.path().to_path_buf();
    fs::create_dir_all(gitdir.join("info")).unwrap();
    // Trailing spaces around an entry should still be recognised as a match.
    fs::write(
        gitdir.join("info").join("exclude"),
        "  .claude-profile.json  \n",
    )
    .unwrap();

    assert!(!ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());
}
