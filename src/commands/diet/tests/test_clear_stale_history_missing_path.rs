use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_missing_path() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/exists/proj": {"history": ["cmd1"]}
        }
    });

    // Path not in JSON — no error.
    let stale_paths = vec!["/nonexistent/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Existing project should be untouched.
    let history = parsed["projects"]["/exists/proj"]["history"].as_array().unwrap();
    assert_eq!(history.len(), 1);
}
