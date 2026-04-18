use super::super::*;
use std::path::Path;

#[test]
fn test_parse_config_dir_only() {
    let json = br#"{"config_dir": "~/.claude-custom"}"#;
    let pf = parse(json, Path::new("test.json")).unwrap();
    assert!(pf.name.is_none());
    assert_eq!(pf.config_dir.as_deref(), Some("~/.claude-custom"));
}
