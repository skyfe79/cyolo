use super::super::*;

/// Digits outside `[1..=profile_count]` are rejected as `Invalid` (including
/// `1` when the profile list is empty).
#[test]
fn test_parse_menu_input_rejects_out_of_range() {
    assert_eq!(parse_menu_input("0", 3), MenuChoice::Invalid);
    assert_eq!(parse_menu_input("4", 3), MenuChoice::Invalid);
    assert_eq!(parse_menu_input("1", 0), MenuChoice::Invalid);
}
