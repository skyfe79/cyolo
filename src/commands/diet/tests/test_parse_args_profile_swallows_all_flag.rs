use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_profile_swallows_all_flag() {
    let args = vec!["--profile".to_string(), "--all".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
