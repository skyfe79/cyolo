use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_partial() {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path();

    // Only create statsig
    fs::create_dir(claude_home.join("statsig")).unwrap();
    fs::write(claude_home.join("statsig").join("data.json"), "abc").unwrap(); // 3 bytes

    let result = measure_cache_dirs(claude_home);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "statsig");
    assert_eq!(result[0].size, 3);
}
