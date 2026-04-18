use super::super::*;
use super::common::*;

#[test]
fn test_atomic_write_json_overwrites_existing() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("output.json");

    // Write initial content
    fs::write(&path, "old content").unwrap();

    // Overwrite with atomic_write_json
    atomic_write_json(&path, "new content").unwrap();

    let written = fs::read_to_string(&path).unwrap();
    assert_eq!(written, "new content");
}
