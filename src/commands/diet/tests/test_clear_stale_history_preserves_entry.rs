use super::super::*;
use super::common::*;

#[test]
fn test_clear_stale_history_preserves_entry() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/stale/proj": {
                "name": "my-project",
                "history": ["old-cmd"],
                "settings": {"key": "value"}
            }
        }
    });

    let stale_paths = vec!["/stale/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Project entry still exists with other fields preserved.
    let proj = &parsed["projects"]["/stale/proj"];
    assert_eq!(proj["name"], "my-project");
    assert_eq!(proj["settings"]["key"], "value");
    // Only history is cleared.
    assert!(proj["history"].as_array().unwrap().is_empty());
}
