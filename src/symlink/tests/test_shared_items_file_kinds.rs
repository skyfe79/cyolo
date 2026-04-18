use super::super::*;

#[test]
fn test_shared_items_file_kinds() {
    let files: Vec<&str> = SHARED_ITEMS
        .iter()
        .filter(|i| i.kind == ItemKind::File)
        .map(|i| i.name)
        .collect();
    assert_eq!(files, vec!["CLAUDE.md", "settings.json", "settings.local.json"]);
}
