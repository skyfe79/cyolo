use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_ensure_exclude_entry_preserves_existing_content() {
    let tmp = TempDir::new().unwrap();
    let gitdir = tmp.path().to_path_buf();
    fs::create_dir_all(gitdir.join("info")).unwrap();
    fs::write(
        gitdir.join("info").join("exclude"),
        "# git ls-files --others --exclude-from=.git/info/exclude\n*.log\n",
    )
    .unwrap();

    assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

    let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
    assert_eq!(
        contents,
        "# git ls-files --others --exclude-from=.git/info/exclude\n*.log\n.claude-profile.json\n"
    );
}
