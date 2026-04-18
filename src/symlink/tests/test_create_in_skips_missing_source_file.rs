use super::super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_create_in_skips_missing_source_file() {
    let source_base = TempDir::new().unwrap();
    let target_base = TempDir::new().unwrap();
    // Only populate directory-kind sources — every file-kind source is
    // intentionally missing so §4.6 rule 2 (skip-and-warn) is exercised.
    for item in SHARED_ITEMS {
        if item.kind == ItemKind::Directory {
            fs::create_dir_all(source_base.path().join(item.name)).unwrap();
        }
    }

    create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

    for item in SHARED_ITEMS {
        let target = target_base.path().join(item.name);
        match item.kind {
            ItemKind::File => {
                assert!(
                    fs::symlink_metadata(&target).is_err(),
                    "file-kind target should not be created when source is missing: {}",
                    item.name
                );
            }
            ItemKind::Directory => {
                assert!(
                    fs::symlink_metadata(&target).is_ok(),
                    "dir-kind target should still be linked: {}",
                    item.name
                );
            }
        }
    }
}
