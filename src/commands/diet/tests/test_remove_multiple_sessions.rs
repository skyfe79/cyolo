use super::super::*;
use super::common::*;

#[test]
fn test_remove_multiple_sessions() {
    let dir = TempDir::new().unwrap();
    let s1 = dir.path().join("s1");
    let s2 = dir.path().join("s2");
    fs::create_dir(&s1).unwrap();
    fs::create_dir(&s2).unwrap();
    fs::write(s1.join("a.txt"), "aaa").unwrap();
    fs::write(s2.join("b.txt"), "bb").unwrap();

    let sessions = vec![
        OrphanedSession {
            folder_path: s1.clone(),
            total_size: 3,
        },
        OrphanedSession {
            folder_path: s2.clone(),
            total_size: 2,
        },
    ];

    let (removed, freed) = remove_session_folders(&sessions).unwrap();

    assert_eq!(removed, 2);
    assert_eq!(freed, 5);
    assert!(!s1.exists());
    assert!(!s2.exists());
}
