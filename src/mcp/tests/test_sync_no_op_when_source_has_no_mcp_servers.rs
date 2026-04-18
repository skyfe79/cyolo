use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_no_op_when_source_has_no_mcp_servers() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.json");
    fs::write(&source, r#"{"unrelated": true}"#).unwrap();

    let profile = tmp.path().join("profile-d");
    assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 0);
    assert!(!profile.join(".claude.json").exists());
}
