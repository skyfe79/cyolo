use super::super::*;

/// Anything that is not a recognized keyword or digit-in-range is `Invalid`.
/// Covers the "typo / accidental input" case for the user.
#[test]
fn test_parse_menu_input_invalid_tokens() {
    assert_eq!(parse_menu_input("x", 2), MenuChoice::Invalid);
    assert_eq!(parse_menu_input("-1", 2), MenuChoice::Invalid);
    assert_eq!(parse_menu_input("1.5", 2), MenuChoice::Invalid);
}
