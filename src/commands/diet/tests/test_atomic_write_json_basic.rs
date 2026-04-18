use super::super::*;
use super::common::*;

#[test]
fn test_atomic_write_json_basic() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("output.json");
    let content = r#"{"key": "value"}"#;

    atomic_write_json(&path, content).unwrap();

    let written = fs::read_to_string(&path).unwrap();
    assert_eq!(written, content);
}
