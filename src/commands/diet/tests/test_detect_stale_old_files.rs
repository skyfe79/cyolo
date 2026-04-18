use super::super::*;
use super::common::*;

#[test]
fn test_detect_stale_old_files() {
    use std::fs::FileTimes;
    use std::time::Duration;

    let projects_dir = TempDir::new().unwrap();
    let project_dir = TempDir::new().unwrap();
    let project_path = project_dir.path().to_string_lossy().to_string();

    // Create session dir with a file that has an old mtime.
    let session_name = project_path_to_session_dir(&project_path);
    let session_path = projects_dir.path().join(&session_name);
    fs::create_dir_all(&session_path).unwrap();

    let file_path = session_path.join("session.jsonl");
    fs::write(&file_path, "some session data").unwrap();

    let stale_days: u32 = 30;
    let old_time = SystemTime::now() - Duration::from_secs(stale_days as u64 * 86400 + 1);
    let file = fs::File::options().write(true).open(&file_path).unwrap();
    file.set_times(FileTimes::new().set_modified(old_time)).unwrap();

    let json: serde_json::Value = serde_json::json!({
        "projects": {
            project_path.clone(): {"history": []}
        }
    });

    let result = detect_stale_projects(&json, projects_dir.path(), stale_days);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].path, project_path);
    assert!(result[0].last_activity_secs >= stale_days as u64 * 86400);
    assert!(result[0].session_size > 0);
}
