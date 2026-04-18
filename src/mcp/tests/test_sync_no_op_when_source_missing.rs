use super::super::*;
use tempfile::TempDir;

#[test]
fn test_sync_no_op_when_source_missing() {
    let tmp = TempDir::new().unwrap();
    let profile = tmp.path().join("profile-c");
    assert_eq!(
        sync_mcp_to_profile_from(&tmp.path().join("absent.json"), &profile).unwrap(),
        0
    );
    assert!(!profile.join(".claude.json").exists());
}
