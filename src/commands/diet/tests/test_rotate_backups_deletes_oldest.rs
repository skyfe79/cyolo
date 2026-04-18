use super::super::*;
use super::common::*;

#[test]
fn test_rotate_backups_deletes_oldest() {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join("claude.json");
    fs::write(&base, "original").unwrap();

    let timestamps = [
        "20260417120001",
        "20260417120002",
        "20260417120003",
        "20260417120004",
        "20260417120005",
        "20260417120006",
        "20260417120007",
    ];
    for ts in &timestamps {
        fs::write(dir.path().join(format!("claude.json.backup-{ts}")), "backup").unwrap();
    }

    rotate_backups(&base, 5).unwrap();

    // Oldest 2 should be deleted
    assert!(!dir.path().join("claude.json.backup-20260417120001").exists());
    assert!(!dir.path().join("claude.json.backup-20260417120002").exists());
    // Newest 5 should remain
    for ts in &timestamps[2..] {
        assert!(dir.path().join(format!("claude.json.backup-{ts}")).exists());
    }
    // Base file untouched
    assert_eq!(fs::read_to_string(&base).unwrap(), "original");
}
