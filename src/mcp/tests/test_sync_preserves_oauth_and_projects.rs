use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_preserves_oauth_and_projects() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.json");
    fs::write(&source, r#"{"mcpServers": {"agent": {"url": "x"}}}"#).unwrap();

    let profile = tmp.path().join("profile-b");
    fs::create_dir_all(&profile).unwrap();
    fs::write(
        profile.join(".claude.json"),
        r#"{"oauthAccount": {"emailAddress": "x@y"}, "projects": {"/foo": {}}}"#,
    )
    .unwrap();

    assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 1);

    let parsed: Value =
        serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap()).unwrap();
    assert_eq!(
        parsed["oauthAccount"]["emailAddress"],
        Value::String("x@y".into())
    );
    assert!(parsed["projects"].as_object().unwrap().contains_key("/foo"));
    assert!(parsed["mcpServers"].as_object().unwrap().contains_key("agent"));
}
