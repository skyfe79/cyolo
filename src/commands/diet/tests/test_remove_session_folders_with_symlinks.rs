use super::super::*;
use super::common::*;

#[test]
fn test_remove_session_folders_with_symlinks() {
    let dir = TempDir::new().unwrap();
    let target_dir = TempDir::new().unwrap();

    // Create a target file that should NOT be deleted
    let target_file = target_dir.path().join("important.txt");
    fs::write(&target_file, "important data").unwrap();

    // Create session folder with a symlink and a regular file
    let session_dir = dir.path().join("session-with-symlink");
    fs::create_dir(&session_dir).unwrap();
    fs::write(session_dir.join("regular.txt"), "regular").unwrap();

    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&target_file, session_dir.join("link.txt")).unwrap();
    }

    let sessions = vec![OrphanedSession {
        folder_path: session_dir.clone(),
        total_size: 100,
    }];

    let (removed, freed) = remove_session_folders(&sessions).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 100);
    assert!(!session_dir.exists());
    // Target file should still exist — symlink was unlinked, not followed
    assert!(target_file.exists());
    assert_eq!(fs::read_to_string(&target_file).unwrap(), "important data");
}
