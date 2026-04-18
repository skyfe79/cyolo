use super::super::*;

/// 1-based digits within range map to 0-based `Pick`; whitespace padding is tolerated.
#[test]
fn test_parse_menu_input_digits() {
    assert_eq!(parse_menu_input("1", 3), MenuChoice::Pick(0));
    assert_eq!(parse_menu_input("3", 3), MenuChoice::Pick(2));
    assert_eq!(parse_menu_input("  2  ", 3), MenuChoice::Pick(1));
}
