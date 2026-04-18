use super::super::*;
use serde_json::json;

/// Contract: the source wins. `~/.claude.json` is the single source of truth
/// for user MCPs; a profile's older `mcpServers` entry is stale.
#[test]
fn test_merge_overrides_existing_mcp_servers() {
    let target = json!({"mcpServers": {"old": {}}});
    let merged = merge_mcp_servers(target, json!({"agent": {}}));
    assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
}
