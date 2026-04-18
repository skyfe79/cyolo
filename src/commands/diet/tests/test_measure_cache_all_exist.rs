use super::super::*;
use super::common::*;

#[test]
fn test_measure_cache_all_exist() {
    let dir = TempDir::new().unwrap();
    let claude_home = dir.path();

    // Create all three cache directories with files
    fs::create_dir(claude_home.join("statsig")).unwrap();
    fs::write(claude_home.join("statsig").join("data.json"), "statsig data").unwrap(); // 12 bytes

    fs::create_dir(claude_home.join("shell-snapshots")).unwrap();
    fs::write(claude_home.join("shell-snapshots").join("snap.bin"), "snapshot").unwrap(); // 8 bytes

    fs::create_dir(claude_home.join("file-history")).unwrap();
    fs::write(claude_home.join("file-history").join("history.log"), "hist").unwrap(); // 4 bytes

    let result = measure_cache_dirs(claude_home);

    assert_eq!(result.len(), 3);
    assert_eq!(result[0].name, "statsig");
    assert_eq!(result[0].size, 12);
    assert_eq!(result[1].name, "shell-snapshots");
    assert_eq!(result[1].size, 8);
    assert_eq!(result[2].name, "file-history");
    assert_eq!(result[2].size, 4);
}
