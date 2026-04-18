use super::super::*;
use super::common::*;

#[test]
fn test_rotate_backups_keeps_all_when_under_limit() {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join("claude.json");
    fs::write(&base, "original").unwrap();

    for ts in &["20260417120001", "20260417120002", "20260417120003"] {
        fs::write(dir.path().join(format!("claude.json.backup-{ts}")), "backup").unwrap();
    }

    rotate_backups(&base, 5).unwrap();

    // All 3 should still exist
    for ts in &["20260417120001", "20260417120002", "20260417120003"] {
        assert!(dir.path().join(format!("claude.json.backup-{ts}")).exists());
    }
}
