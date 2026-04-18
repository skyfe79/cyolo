use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_non_array() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/bad/proj": {"history": "not-an-array"}
        }
    });

    let stale_paths = vec!["/bad/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // history should remain unchanged (not an array, so skipped).
    assert_eq!(parsed["projects"]["/bad/proj"]["history"], "not-an-array");
}
