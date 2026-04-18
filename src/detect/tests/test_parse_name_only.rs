use super::super::*;
use std::path::Path;

#[test]
fn test_parse_name_only() {
    let json = br#"{"name": "work"}"#;
    let pf = parse(json, Path::new("test.json")).unwrap();
    assert_eq!(pf.name.as_deref(), Some("work"));
    assert!(pf.config_dir.is_none());
}
