use super::super::*;
use super::common::*;

#[test]
fn test_scan_missing_projects_dir() {
    let dir = TempDir::new().unwrap();
    let missing = dir.path().join("nonexistent");
    let orphaned_paths = vec!["/some/path".to_string()];
    let (sessions, total) = scan_session_folders(&missing, &orphaned_paths);
    assert!(sessions.is_empty());
    assert_eq!(total, 0);
}
