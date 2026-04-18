use super::super::*;
use super::common::*;

#[test]
fn test_scan_ignores_file_as_session() {
    let dir = TempDir::new().unwrap();
    let projects_dir = dir.path();

    // Create a regular file (not a directory) with the encoded session name
    let session_name = project_path_to_session_dir("/Users/codingmax/fake-project");
    fs::write(projects_dir.join(&session_name), "not a dir").unwrap();

    let orphaned_paths = vec!["/Users/codingmax/fake-project".to_string()];
    let (sessions, _total) = scan_session_folders(projects_dir, &orphaned_paths);

    // Should NOT be included because it's a file, not a directory
    assert!(sessions.is_empty());
}
