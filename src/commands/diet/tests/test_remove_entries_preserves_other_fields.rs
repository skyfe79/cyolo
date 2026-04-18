use super::super::*;
use super::common::*;

#[test]
fn test_remove_entries_preserves_other_fields() {
    let content = r#"{
  "version": "1.0",
  "projects": {
"/active/project": {"name": "active"},
"/orphan/one": {"name": "orphan1"},
"/orphan/two": {"name": "orphan2"}
  },
  "settings": {"theme": "dark"}
}"#;
    let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    let orphaned = vec!["/orphan/one".to_string(), "/orphan/two".to_string()];

    remove_orphaned_entries(&mut parsed, &orphaned);

    // Verify in-memory mutation
    let projects = parsed["projects"].as_object().unwrap();
    assert_eq!(projects.len(), 1);
    assert!(projects.contains_key("/active/project"));
    assert!(!projects.contains_key("/orphan/one"));
    assert!(!projects.contains_key("/orphan/two"));
    // Other top-level fields preserved
    assert_eq!(parsed["version"], "1.0");
    assert_eq!(parsed["settings"]["theme"], "dark");
}
