use super::super::*;
use tempfile::TempDir;

#[test]
fn test_config_dir_in_joins_cyolo() {
    let tmp = TempDir::new().unwrap();
    assert_eq!(config_dir_in(tmp.path()), tmp.path().join(".cyolo"));
}
