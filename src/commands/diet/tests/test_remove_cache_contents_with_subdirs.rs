use super::super::*;
use super::common::*;

#[test]
fn test_remove_cache_contents_with_subdirs() {
    let dir = TempDir::new().unwrap();
    let cache_path = dir.path().join("file-history");
    fs::create_dir(&cache_path).unwrap();
    fs::write(cache_path.join("top.txt"), "top").unwrap();
    fs::create_dir(cache_path.join("nested")).unwrap();
    fs::write(cache_path.join("nested").join("deep.txt"), "deep").unwrap();
    fs::create_dir(cache_path.join("nested").join("deeper")).unwrap();
    fs::write(
        cache_path.join("nested").join("deeper").join("file.bin"),
        "bin",
    )
    .unwrap();

    let caches = vec![CacheDir {
        name: "file-history".to_string(),
        path: cache_path.clone(),
        size: 10,
    }];

    let (removed, freed) = remove_cache_contents(&caches).unwrap();

    assert_eq!(removed, 1);
    assert_eq!(freed, 10);
    assert!(cache_path.exists());
    let entries: Vec<_> = fs::read_dir(&cache_path).unwrap().collect();
    assert!(entries.is_empty());
}
