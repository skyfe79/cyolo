use super::super::*;
use serde_json::json;

/// Corrupt or stub `.claude.json` (e.g. an array) should not tank the sync —
/// we degrade to a fresh object carrying just mcpServers.
#[test]
fn test_merge_replaces_non_object_target_root() {
    let merged = merge_mcp_servers(json!([1, 2, 3]), json!({"agent": {}}));
    assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
}
