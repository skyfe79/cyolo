use super::super::*;

/// `d` / `default`, case-insensitive. Works even when no profiles are
/// registered — the option is always available and never index-sensitive.
#[test]
fn test_parse_menu_input_default_aliases() {
    assert_eq!(parse_menu_input("d", 2), MenuChoice::Default);
    assert_eq!(parse_menu_input("D", 2), MenuChoice::Default);
    assert_eq!(parse_menu_input("default", 2), MenuChoice::Default);
    assert_eq!(parse_menu_input("DEFAULT", 2), MenuChoice::Default);
    assert_eq!(parse_menu_input("d", 0), MenuChoice::Default);
}
