use super::super::*;
use super::common::*;

#[test]
fn test_analyze_mixed() {
    // Create a real directory that "exists"
    let real_dir = TempDir::new().unwrap();
    let real_path = real_dir.path().to_string_lossy().to_string();

    let dir = TempDir::new().unwrap();
    let content = format!(
        r#"{{"projects": {{
            "{}": {{"name": "active"}},
            "/nonexistent/orphan/one": {{"name": "orphan1"}},
            "/nonexistent/orphan/two": {{"name": "orphan2"}}
        }}}}"#,
        real_path
    );
    let path = write_claude_json(&dir, &content);

    let result = analyze_claude_json(&path).unwrap();

    assert_eq!(result.orphaned_projects.len(), 2);
    assert_eq!(result.active_count, 1);
    assert!(result.config_file_size > 0);
    // parsed_json should have the projects key
    assert!(result.parsed_json.get("projects").is_some());
}
