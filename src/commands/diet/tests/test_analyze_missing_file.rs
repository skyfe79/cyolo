use super::super::*;
use super::common::*;

#[test]
fn test_analyze_missing_file() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nonexistent.json");

    let result = analyze_claude_json(&path).unwrap();

    assert!(result.orphaned_projects.is_empty());
    assert_eq!(result.active_count, 0);
    assert_eq!(result.config_file_size, 0);
    assert_eq!(result.parsed_json, serde_json::Value::Null);
}
