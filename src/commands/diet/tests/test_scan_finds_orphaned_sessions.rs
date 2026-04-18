use super::super::*;
use super::common::*;

#[test]
fn test_scan_finds_orphaned_sessions() {
    let dir = TempDir::new().unwrap();
    let projects_dir = dir.path();

    // Create session folders matching orphaned paths
    let session_name =
        project_path_to_session_dir("/Users/codingmax/Private/labs/test-bot");
    let session_path = projects_dir.join(&session_name);
    fs::create_dir(&session_path).unwrap();
    fs::write(session_path.join("data.json"), "test data here").unwrap(); // 14 bytes

    // Also create a non-orphaned session folder
    let active_name =
        project_path_to_session_dir("/Users/codingmax/active-project");
    let active_path = projects_dir.join(&active_name);
    fs::create_dir(&active_path).unwrap();
    fs::write(active_path.join("state.json"), "active").unwrap(); // 6 bytes

    let orphaned_paths =
        vec!["/Users/codingmax/Private/labs/test-bot".to_string()];
    let (sessions, total) = scan_session_folders(projects_dir, &orphaned_paths);

    assert_eq!(sessions.len(), 1);
    assert_eq!(sessions[0].folder_path, session_path);
    assert_eq!(sessions[0].total_size, 14);
    // total should include ALL session dirs (orphaned + active)
    assert_eq!(total, 20); // 14 + 6
}
