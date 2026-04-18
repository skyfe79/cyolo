use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_ignores_file_as_dir() {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path();

    // Create a regular file named "statsig" instead of a directory
    fs::write(claude_home.join("statsig"), "not a dir").unwrap();

    let result = measure_cache_dirs(claude_home);

    assert!(result.is_empty(), "regular file with cache name should be skipped");
}
