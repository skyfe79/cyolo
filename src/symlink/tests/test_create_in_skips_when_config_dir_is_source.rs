use super::super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_create_in_skips_when_config_dir_is_source() {
    let source_base = TempDir::new().unwrap();
    for item in SHARED_ITEMS {
        let src = source_base.path().join(item.name);
        match item.kind {
            ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
            ItemKind::File => fs::write(&src, b"stub").unwrap(),
        }
    }

    // Snapshot children before the call so we can assert nothing changed.
    let mut before: Vec<_> = fs::read_dir(source_base.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    before.sort();

    // config_dir == source_base → §5.1 early-return, no symlinks touched.
    create_shared_symlinks_in(source_base.path(), source_base.path()).unwrap();

    let mut after: Vec<_> = fs::read_dir(source_base.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    after.sort();
    assert_eq!(before, after, "no new entries should be created when source == target");

    // Sanity: none of the entries should be a symlink (all were seeded as
    // regular files/dirs above).
    for item in SHARED_ITEMS {
        let meta = fs::symlink_metadata(source_base.path().join(item.name)).unwrap();
        assert!(!meta.file_type().is_symlink(), "{} became a symlink", item.name);
    }
}
