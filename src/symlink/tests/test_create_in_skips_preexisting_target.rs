use super::super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_create_in_skips_preexisting_target() {
    let source_base = TempDir::new().unwrap();
    let target_base = TempDir::new().unwrap();
    for item in SHARED_ITEMS {
        let src = source_base.path().join(item.name);
        match item.kind {
            ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
            ItemKind::File => fs::write(&src, b"stub").unwrap(),
        }
    }

    // Pre-existing regular file at one target must not be overwritten.
    let preexisting = target_base.path().join("CLAUDE.md");
    fs::write(&preexisting, b"pre-existing content").unwrap();

    create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

    let meta = fs::symlink_metadata(&preexisting).unwrap();
    assert!(
        meta.file_type().is_file(),
        "pre-existing regular file must not be replaced by a symlink (§4.6 rule 3)"
    );
    assert_eq!(fs::read(&preexisting).unwrap(), b"pre-existing content");
}
