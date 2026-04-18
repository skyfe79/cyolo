use super::super::*;
use super::common::*;

#[test]
fn test_detect_stale_orphan_skipped() {
    let projects_dir = TempDir::new().unwrap();

    // Project path does NOT exist on disk.
    let nonexistent_path = "/nonexistent/path/to/project";

    let json: serde_json::Value = serde_json::json!({
        "projects": {
            nonexistent_path: {"history": []}
        }
    });

    let result = detect_stale_projects(&json, projects_dir.path(), 30);

    assert!(result.is_empty(), "nonexistent project path should be skipped entirely");
}
