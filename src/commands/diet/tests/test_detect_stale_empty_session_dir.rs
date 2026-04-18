use super::super::*;
use super::common::*;

#[test]
fn test_detect_stale_empty_session_dir() {
    let projects_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let project_path = project_dir.path().to_string_lossy().to_string();

    // Create an empty session directory.
    let session_name = project_path_to_session_dir(&project_path);
    let session_path = projects_dir.path().join(&session_name);
    fs::create_dir_all(&session_path).unwrap();

    let json: serde_json::Value = serde_json::json!({
        "projects": {
            project_path.clone(): {"history": []}
        }
    });

    let result = detect_stale_projects(&json, projects_dir.path(), 30);

    assert!(result.is_empty(), "project with empty session dir should not be stale");
}
