use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_no_projects_key() {
    let mut parsed: serde_json::Value = serde_json::json!({"version": "1.0"});
    let stale_paths = vec!["/some/path".to_string()];

    // Should not panic or error.
    clear_stale_history(&mut parsed, &stale_paths);

    assert_eq!(parsed["version"], "1.0");
}
