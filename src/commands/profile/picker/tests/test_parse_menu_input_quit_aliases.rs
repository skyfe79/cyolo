use super::super::*;

/// `q` / `quit` plus empty/whitespace-only input — all mean "do nothing".
#[test]
fn test_parse_menu_input_quit_aliases() {
    assert_eq!(parse_menu_input("q", 2), MenuChoice::Quit);
    assert_eq!(parse_menu_input("quit", 2), MenuChoice::Quit);
    assert_eq!(parse_menu_input("", 2), MenuChoice::Quit);
    assert_eq!(parse_menu_input("   ", 2), MenuChoice::Quit);
}
