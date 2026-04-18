use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_profile_swallows_flag() {
    // Regression: --profile --apply should error, not set profile="--apply"
    let args = vec!["--profile".to_string(), "--apply".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
