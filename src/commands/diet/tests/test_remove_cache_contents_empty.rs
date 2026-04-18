use super::super::*;
use super::common::*;

#[test]
fn test_remove_cache_contents_empty() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("empty-cache");
    fs::create_dir(&cache_path).unwrap();

    let caches = vec![CacheDir {
        name: "empty-cache".to_string(),
        path: cache_path.clone(),
        size: 0,
    }];

    let (removed, freed) = remove_cache_contents(&caches).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 0);
    assert!(cache_path.exists());
}
