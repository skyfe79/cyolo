use super::super::*;
use super::common::*;

#[test]
fn test_remove_entries_no_projects_key() {
    let content = r#"{"version": "1.0"}"#;
    let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    let orphaned = vec!["/orphan/one".to_string()];

    // Should succeed without error (no-op)
    remove_orphaned_entries(&mut parsed, &orphaned);
}
