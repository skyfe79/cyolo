//! Each sibling file owns exactly one `#[test]` concept. Split by the
//! input category the test exercises (digit, new-aliases, default-aliases,
//! quit-aliases, invalid, out-of-range).

mod test_parse_menu_input_digits;
mod test_parse_menu_input_new_aliases;
mod test_parse_menu_input_default_aliases;
mod test_parse_menu_input_quit_aliases;
mod test_parse_menu_input_rejects_out_of_range;
mod test_parse_menu_input_invalid_tokens;
