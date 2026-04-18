use super::super::*;
use std::path::Path;

#[test]
fn test_parse_both() {
    let json = br#"{"name": "x", "config_dir": "y"}"#;
    let pf = parse(json, Path::new("test.json")).unwrap();
    assert_eq!(pf.name.as_deref(), Some("x"));
    assert_eq!(pf.config_dir.as_deref(), Some("y"));
}
