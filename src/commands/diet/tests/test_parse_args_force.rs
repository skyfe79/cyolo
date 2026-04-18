use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_force() {
    let args = vec!["--force".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert!(!result.apply);
    assert!(result.force);
}
