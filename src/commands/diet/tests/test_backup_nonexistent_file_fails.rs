use super::super::*;
use super::common::*;

#[test]
fn test_backup_nonexistent_file_fails() {
    let dir = TempDir::new().unwrap();
    let json_path = dir.path().join("nonexistent.json");

    let result = backup_claude_json(&json_path);
    assert!(result.is_err());
}
