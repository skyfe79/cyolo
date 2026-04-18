use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_unknown_positional() {
    let args = vec!["cleanup".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
