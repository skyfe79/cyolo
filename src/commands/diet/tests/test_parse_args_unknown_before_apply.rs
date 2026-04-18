use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_unknown_before_apply() {
    // First unknown arg should cause immediate error, even if --apply follows
    let args = vec!["--verbose".to_string(), "--apply".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
