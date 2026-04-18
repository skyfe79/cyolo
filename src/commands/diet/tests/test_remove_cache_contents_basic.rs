use super::super::*;
use super::common::*;

#[test]
fn test_remove_cache_contents_basic() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("statsig");
    fs::create_dir(&cache_path).unwrap();
    fs::write(cache_path.join("file1.json"), "data1").unwrap();
    fs::write(cache_path.join("file2.json"), "data2").unwrap();

    let caches = vec![CacheDir {
        name: "statsig".to_string(),
        path: cache_path.clone(),
        size: 10,
    }];

    let (removed, freed) = remove_cache_contents(&caches).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 10);
    // Cache directory itself should still exist
    assert!(cache_path.exists());
    // But should be empty
    let entries: Vec<_> = fs::read_dir(&cache_path).unwrap().collect();
    assert!(entries.is_empty());
}
