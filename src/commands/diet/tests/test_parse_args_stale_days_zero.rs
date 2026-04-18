use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_stale_days_zero() {
    let args = vec!["--stale-days".to_string(), "0".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
