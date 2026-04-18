use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_none_exist() {
    let dir = TempDir::new().unwrap();
    let result = measure_cache_dirs(dir.path());
    assert!(result.is_empty());
}
