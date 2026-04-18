use super::super::*;
use std::path::Path;

#[test]
fn test_parse_empty_object() {
    let json = br#"{}"#;
    let err = parse(json, Path::new("test.json")).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("expected 'name' or 'config_dir' field"), "got: {msg}");
}
