use super::super::*;
use super::common::*;

#[test]
fn test_remove_session_folder_symlinked_root() {
    // Regression: if the session folder itself is a symlink to an external
    // directory, we must unlink the symlink — never read_dir or remove_dir_all
    // through it, which would mutate data outside ~/.claude/projects/.
    let dir = TempDir::new().unwrap();
    let external_dir = TempDir::new().unwrap();

    // Create files inside the external directory
    let external_file = external_dir.path().join("precious.txt");
    fs::write(&external_file, "do not delete").unwrap();

    // Create a symlink session folder pointing to the external dir
    let session_link = dir.path().join("session-symlink");
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(external_dir.path(), &session_link).unwrap();
    }

    let sessions = vec![OrphanedSession {
        folder_path: session_link.clone(),
        total_size: 50,
    }];

    let (removed, freed) = remove_session_folders(&sessions).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 50);
    // The symlink itself should be gone
    assert!(!session_link.exists());
    // The external directory and its contents must be untouched
    assert!(external_dir.path().exists());
    assert!(external_file.exists());
    assert_eq!(fs::read_to_string(&external_file).unwrap(), "do not delete");
}
