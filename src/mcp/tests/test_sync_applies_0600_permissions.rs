use super::super::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_sync_applies_0600_permissions() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.json");
    fs::write(&source, r#"{"mcpServers": {"agent": {"url": "x"}}}"#).unwrap();

    let profile = tmp.path().join("profile-f");
    sync_mcp_to_profile_from(&source, &profile).unwrap();

    let mode = fs::metadata(profile.join(".claude.json"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        mode & 0o777,
        0o600,
        "expected 0o600, got {:o}",
        mode & 0o777
    );
}
