use super::super::*;
use super::common::*;

#[test]
fn test_atomic_write_json_no_leftover_tmp() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("output.json");
    let tmp_path = path.with_extension("json.tmp");

    atomic_write_json(&path, "test content").unwrap();

    // The .json.tmp file should not remain after a successful write
    assert!(!tmp_path.exists());
    assert!(path.exists());
}
