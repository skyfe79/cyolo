use super::super::*;
use super::common::*;

#[test]
fn test_rotate_backups_no_backups() {
    let dir = TempDir::new().unwrap();
    let base = dir.path().join("claude.json");
    fs::write(&base, "original").unwrap();

    // No backup files exist — should succeed silently
    rotate_backups(&base, 5).unwrap();
}
