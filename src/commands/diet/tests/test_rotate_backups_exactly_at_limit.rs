use super::super::*;
use super::common::*;

#[test]
fn test_rotate_backups_exactly_at_limit() {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join("claude.json");
    fs::write(&base, "original").unwrap();

    for i in 1..=5 {
        fs::write(dir.path().join(format!("claude.json.backup-2026041712000{i}")), "backup").unwrap();
    }

    rotate_backups(&base, 5).unwrap();

    // All 5 should still exist
    for i in 1..=5 {
        assert!(dir.path().join(format!("claude.json.backup-2026041712000{i}")).exists());
    }
}
