use super::super::*;
use super::common::*;

#[test]
fn test_scan_no_matching_sessions() {
    let dir = TempDir::new().unwrap();
    let projects_dir = dir.path();

    // Create a session folder that does NOT match any orphaned path
    let other_name = project_path_to_session_dir("/some/other/project");
    fs::create_dir(projects_dir.join(&other_name)).unwrap();

    let orphaned_paths = vec!["/Users/nonexistent/path".to_string()];
    let (sessions, total) = scan_session_folders(projects_dir, &orphaned_paths);

    assert!(sessions.is_empty());
    // total should still count the existing session folder
    assert_eq!(total, 0); // empty dir has size 0
}
