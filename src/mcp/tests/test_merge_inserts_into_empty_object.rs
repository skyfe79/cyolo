use super::super::*;
use serde_json::json;

#[test]
fn test_merge_inserts_into_empty_object() {
    let merged = merge_mcp_servers(json!({}), json!({"agent": {}}));
    assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
}
