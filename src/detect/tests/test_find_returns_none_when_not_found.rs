use super::super::*;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_find_returns_none_when_not_found() {
    let tmp = TempDir::new().unwrap();
    let child = tmp.path().join("a").join("b");
    fs::create_dir_all(&child).unwrap();

    // Bounded walk-up: guaranteed no .claude-profile.json within tmp.
    let result = walk_up_search(&child, Some(tmp.path())).unwrap();
    assert!(result.is_none());
}
