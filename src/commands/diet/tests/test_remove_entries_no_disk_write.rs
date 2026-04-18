use super::super::*;
use super::common::*;

#[test]
fn test_remove_entries_no_disk_write() {
    // Verify that remove_orphaned_entries is purely in-memory:
    // mutate the JSON, then check the in-memory value (no file path needed).
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/keep/this": {"name": "keeper"},
            "/remove/this": {"name": "goner"}
        }
    });
    let orphaned = vec!["/remove/this".to_string()];

    remove_orphaned_entries(&mut parsed, &orphaned);

    let projects = parsed["projects"].as_object().unwrap();
    assert_eq!(projects.len(), 1);
    assert!(projects.contains_key("/keep/this"));
    assert!(!projects.contains_key("/remove/this"));
}
