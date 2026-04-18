use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_apply_then_unknown() {
    // --apply followed by unknown should still fail (validate all args)
    let args = vec!["--apply".to_string(), "--unknown".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
