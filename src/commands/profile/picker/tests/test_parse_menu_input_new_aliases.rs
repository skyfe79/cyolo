use super::super::*;

/// `n` / `new`, case-insensitive, all map to `MenuChoice::New`.
#[test]
fn test_parse_menu_input_new_aliases() {
    assert_eq!(parse_menu_input("n", 2), MenuChoice::New);
    assert_eq!(parse_menu_input("N", 2), MenuChoice::New);
    assert_eq!(parse_menu_input("new", 2), MenuChoice::New);
    assert_eq!(parse_menu_input("NEW", 2), MenuChoice::New);
}
