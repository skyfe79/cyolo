use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_sync_overrides_stale_target_mcp_but_keeps_other_keys() {
    let tmp = TempDir::new().unwrap();
    let source = tmp.path().join("source.json");
    fs::write(&source, r#"{"mcpServers": {"agent": {}}}"#).unwrap();

    let profile = tmp.path().join("profile-g");
    fs::create_dir_all(&profile).unwrap();
    fs::write(
        profile.join(".claude.json"),
        r#"{"oauthAccount": {"emailAddress": "x@y"}, "mcpServers": {"stale": {}}}"#,
    )
    .unwrap();

    assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 1);

    let parsed: Value =
        serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap()).unwrap();
    // Source won — "stale" is gone, "agent" is present.
    assert!(parsed["mcpServers"].as_object().unwrap().contains_key("agent"));
    assert!(
        !parsed["mcpServers"].as_object().unwrap().contains_key("stale"),
        "expected stale entry to be evicted by source override"
    );
    // OAuth is untouched.
    assert_eq!(
        parsed["oauthAccount"]["emailAddress"],
        Value::String("x@y".into())
    );
}
