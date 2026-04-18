use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_no_history_key() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/no-history/proj": {"name": "proj-without-history"}
        }
    });

    let stale_paths = vec!["/no-history/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Project should be untouched (no history key to clear).
    assert_eq!(parsed["projects"]["/no-history/proj"]["name"], "proj-without-history");
    assert!(parsed["projects"]["/no-history/proj"].get("history").is_none());
}
