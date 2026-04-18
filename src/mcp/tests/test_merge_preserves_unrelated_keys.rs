use super::super::*;
use serde_json::json;

#[test]
fn test_merge_preserves_unrelated_keys() {
    let target = json!({"oauthAccount": {"emailAddress": "user@example.com"}, "projects": {}});
    let merged = merge_mcp_servers(target, json!({"agent": {}}));
    assert_eq!(
        merged,
        json!({
            "oauthAccount": {"emailAddress": "user@example.com"},
            "projects": {},
            "mcpServers": {"agent": {}}
        })
    );
}
