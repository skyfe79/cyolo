use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_all() {
    let args = vec!["--all".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert!(result.all);
}
