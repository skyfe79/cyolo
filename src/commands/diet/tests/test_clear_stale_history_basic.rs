use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_basic() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/stale/one": {"history": ["cmd1", "cmd2", "cmd3"]},
            "/active/two": {"history": ["recent"]}
        }
    });

    let stale_paths = vec!["/stale/one".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Stale path's history should be cleared.
    let stale_history = parsed["projects"]["/stale/one"]["history"].as_array().unwrap();
    assert!(stale_history.is_empty());

    // Active path's history should be untouched.
    let active_history = parsed["projects"]["/active/two"]["history"].as_array().unwrap();
    assert_eq!(active_history.len(), 1);
}
