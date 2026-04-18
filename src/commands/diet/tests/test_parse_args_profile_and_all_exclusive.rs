use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_profile_and_all_exclusive() {
    let args = vec!["--profile".to_string(), "x".to_string(), "--all".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
