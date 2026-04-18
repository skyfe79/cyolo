use super::super::*;
use super::common::*;

#[test]
fn test_scan_skips_symlinked_session_root() {
    // If a session folder entry is a symlink, scan_session_folders should skip it.
    let dir = TempDir::new().unwrap();
    let external_dir = TempDir::new().unwrap();
    fs::write(external_dir.path().join("data.txt"), "external").unwrap();

    let session_name = project_path_to_session_dir("/Users/codingmax/symlinked-proj");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(external_dir.path(), dir.path().join(&session_name))
            .unwrap();
    }

    let orphaned_paths = vec!["/Users/codingmax/symlinked-proj".to_string()];
    let (sessions, _total) = scan_session_folders(dir.path(), &orphaned_paths);

    // The symlinked session folder should NOT be included
    assert!(sessions.is_empty());
}
