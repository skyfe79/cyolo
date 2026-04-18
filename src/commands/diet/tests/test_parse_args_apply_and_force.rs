use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_apply_and_force() {
    let args = vec!["--apply".to_string(), "--force".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert!(result.apply);
    assert!(result.force);
}
