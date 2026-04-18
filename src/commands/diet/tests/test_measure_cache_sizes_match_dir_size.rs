use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_sizes_match_dir_size() {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path();

    fs::create_dir(claude_home.join("statsig")).unwrap();
    fs::write(claude_home.join("statsig").join("a.bin"), vec![0u8; 100]).unwrap();
    fs::create_dir(claude_home.join("statsig").join("sub")).unwrap();
    fs::write(
        claude_home.join("statsig").join("sub").join("b.bin"),
        vec![0u8; 50],
    )
    .unwrap();

    let result = measure_cache_dirs(claude_home);

    assert_eq!(result.len(), 1);
    let expected_size = dir_size(&claude_home.join("statsig"));
    assert_eq!(result[0].size, expected_size);
    assert_eq!(result[0].size, 150);
}
