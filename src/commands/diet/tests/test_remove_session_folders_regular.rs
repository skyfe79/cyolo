use super::super::*;
use super::common::*;

#[test]
fn test_remove_session_folders_regular() {
    let dir = TempDir::new().unwrap();
    let session_dir = dir.path().join("session1");
    fs::create_dir(&session_dir).unwrap();
    fs::write(session_dir.join("data.json"), "test data").unwrap(); // 9 bytes
    fs::create_dir(session_dir.join("sub")).unwrap();
    fs::write(session_dir.join("sub").join("nested.txt"), "nested").unwrap(); // 6 bytes

    let sessions = vec![OrphanedSession {
        folder_path: session_dir.clone(),
        total_size: 15,
    }];

    let (removed, freed) = remove_session_folders(&sessions).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 15);
    assert!(!session_dir.exists());
}
