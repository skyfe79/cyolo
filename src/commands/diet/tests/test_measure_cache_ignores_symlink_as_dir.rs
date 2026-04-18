use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_ignores_symlink_as_dir() {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path();
    let target_dir = TempDir::new().unwrap();

    // Create a symlink named "statsig" → should be skipped
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(target_dir.path(), claude_home.join("statsig")).unwrap();
    }

    let result = measure_cache_dirs(claude_home);

    assert!(result.is_empty(), "symlinked cache dir should be skipped");
}
