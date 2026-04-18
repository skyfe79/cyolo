use super::super::*;
use super::common::*;

#[test]
fn test_analyze_empty_projects() {
    let dir = TempDir::new().unwrap();
    let path = write_claude_json(&dir, r#"{"projects": {}}"#);

    let result = analyze_claude_json(&path).unwrap();

    assert!(result.orphaned_projects.is_empty());
    assert_eq!(result.active_count, 0);
    assert!(result.config_file_size > 0);
}
