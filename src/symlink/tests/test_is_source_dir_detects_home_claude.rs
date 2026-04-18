use super::super::*;

#[test]
fn test_is_source_dir_detects_home_claude() {
    if let Some(home) = dirs::home_dir() {
        assert!(is_source_dir(&home.join(".claude")));
    }
}
