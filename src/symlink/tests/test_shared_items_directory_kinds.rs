use super::super::*;

#[test]
fn test_shared_items_directory_kinds() {
    let dirs: Vec<&str> = SHARED_ITEMS
        .iter()
        .filter(|i| i.kind == ItemKind::Directory)
        .map(|i| i.name)
        .collect();
    assert_eq!(dirs, vec!["commands", "skills", "agents"]);
}
