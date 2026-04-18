use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_creates_file_when_absent() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.json");
    fs::write(
        &source,
        r#"{"mcpServers": {"agent": {"url": "x"}, "github": {"url": "y"}}}"#,
    )
    .unwrap();

    let profile = tmp.path().join("profile-a");
    let written = sync_mcp_to_profile_from(&source, &profile).unwrap();
    assert_eq!(written, 2);

    let parsed: Value =
        serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap()).unwrap();
    assert_eq!(
        parsed["mcpServers"]["agent"]["url"],
        Value::String("x".into())
    );
    assert_eq!(
        parsed["mcpServers"]["github"]["url"],
        Value::String("y".into())
    );
}
