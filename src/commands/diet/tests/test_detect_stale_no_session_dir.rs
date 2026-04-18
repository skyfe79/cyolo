use super::super::*;
use super::common::*;

#[test]
fn test_detect_stale_no_session_dir() {
    let projects_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let project_path = project_dir.path().to_string_lossy().to_string();

    // No session directory created for this project.
    let json: serde_json::Value = serde_json::json!({
        "projects": {
            project_path.clone(): {"history": []}
        }
    });

    let result = detect_stale_projects(&json, projects_dir.path(), 30);

    assert!(result.is_empty(), "project with no session dir should not be stale");
}
