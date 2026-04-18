use super::super::*;
use super::common::*;

#[test]
fn test_remove_cache_contents_multiple() {
    let dir = TempDir::new().unwrap();

    let c1_path = dir.path().join("c1");
    fs::create_dir(&c1_path).unwrap();
    fs::write(c1_path.join("f1.txt"), "aaa").unwrap();

    let c2_path = dir.path().join("c2");
    fs::create_dir(&c2_path).unwrap();
    fs::write(c2_path.join("f2.txt"), "bb").unwrap();

    let caches = vec![
        CacheDir {
            name: "c1".to_string(),
            path: c1_path.clone(),
            size: 3,
        },
        CacheDir {
            name: "c2".to_string(),
            path: c2_path.clone(),
            size: 2,
        },
    ];

    let (removed, freed) = remove_cache_contents(&caches).unwrap();

    assert_eq!(removed, 2);
    assert_eq!(freed, 5);
    assert!(c1_path.exists());
    assert!(c2_path.exists());
    assert!(fs::read_dir(&c1_path).unwrap().count() == 0);
    assert!(fs::read_dir(&c2_path).unwrap().count() == 0);
}
