use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_in_ancestor() {
    let tmp = TempDir::new().unwrap();
    let profile_path = tmp.path().join(PROFILE_FILENAME);
    fs::write(&profile_path, r#"{"config_dir": "/custom"}"#).unwrap();

    let child = tmp.path().join("sub").join("deep");
    fs::create_dir_all(&child).unwrap();

    let result = walk_up_search(&child, Some(tmp.path())).unwrap();
    assert!(result.is_some());
    let (path, pf) = result.unwrap();
    assert_eq!(path, profile_path);
    assert_eq!(pf.config_dir.as_deref(), Some("/custom"));
}
