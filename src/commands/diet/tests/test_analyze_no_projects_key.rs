use super::super::*;
use super::common::*;

#[test]
fn test_analyze_no_projects_key() {
    let dir = TempDir::new().unwrap();
    let path = write_claude_json(&dir, r#"{"version": "1.0"}"#);

    let result = analyze_claude_json(&path).unwrap();

    assert!(result.orphaned_projects.is_empty());
    assert_eq!(result.active_count, 0);
    assert!(result.config_file_size > 0);
    assert!(result.parsed_json.get("version").is_some());
}
