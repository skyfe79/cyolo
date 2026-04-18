use super::super::*;
use tempfile::TempDir;

#[test]
fn test_config_path_in_joins_config_json() {
    let tmp = TempDir::new().unwrap();
    assert_eq!(
        config_path_in(tmp.path()),
        tmp.path().join(".cyolo").join("config.json")
    );
}
