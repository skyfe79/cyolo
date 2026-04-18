use super::super::*;
use tempfile::TempDir;

#[cfg(unix)]
#[test]
fn test_create_in_links_existing_source_directory() {
    let source_base = TempDir::new().unwrap();
    let target_base = TempDir::new().unwrap();

    // Pre-populate every shared item so the code path never has to
    // auto-create or warn — this test exercises the happy path only.
    for item in SHARED_ITEMS {
        let src = source_base.path().join(item.name);
        match item.kind {
            ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
            ItemKind::File => fs::write(&src, b"stub").unwrap(),
        }
    }

    create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

    let commands_link = target_base.path().join("commands");
    let resolved = fs::read_link(&commands_link).unwrap().canonicalize().unwrap();
    let expected = source_base.path().join("commands").canonicalize().unwrap();
    assert_eq!(resolved, expected);
}
