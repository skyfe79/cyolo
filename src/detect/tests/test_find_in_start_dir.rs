use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_in_start_dir() {
    let tmp = TempDir::new().unwrap();
    let profile_path = tmp.path().join(PROFILE_FILENAME);
    fs::write(&profile_path, r#"{"name": "local"}"#).unwrap();

    let result = walk_up_search(tmp.path(), Some(tmp.path())).unwrap();
    assert!(result.is_some());
    let (path, pf) = result.unwrap();
    assert_eq!(path, profile_path);
    assert_eq!(pf.name.as_deref(), Some("local"));
}
