use super::super::*;
use super::common::*;

#[test]
fn test_backup_creates_file() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("claude.json");
    fs::write(&json_path, r#"{"projects": {}}"#).unwrap();

    let backup_path = backup_claude_json(&json_path).unwrap();

    assert!(backup_path.exists());
    assert!(backup_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .starts_with("claude.json.backup-"));
    // Backup content matches original
    let original = fs::read_to_string(&json_path).unwrap();
    let backup = fs::read_to_string(&backup_path).unwrap();
    assert_eq!(original, backup);
}
