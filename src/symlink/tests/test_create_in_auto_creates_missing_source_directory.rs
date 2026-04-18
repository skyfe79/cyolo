use super::super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_create_in_auto_creates_missing_source_directory() {
    let source_base = TempDir::new().unwrap();
    let target_base = TempDir::new().unwrap();
    // Pre-create the file-kind sources so the loop only has to auto-create
    // the directory-kind ones.  No dirs pre-exist in `source_base`.
    for item in SHARED_ITEMS {
        if item.kind == ItemKind::File {
            fs::write(source_base.path().join(item.name), b"stub").unwrap();
        }
    }

    create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

    // Directory-kind sources should have been auto-created (§4.6 rule 1).
    let created_source = source_base.path().join("commands");
    assert!(created_source.is_dir(), "source dir should be auto-created");

    let link = target_base.path().join("commands").canonicalize().unwrap();
    let expected = created_source.canonicalize().unwrap();
    assert_eq!(link, expected);
}
