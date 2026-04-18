use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_cache() {
    let args = vec!["--cache".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert!(result.cache);
}
