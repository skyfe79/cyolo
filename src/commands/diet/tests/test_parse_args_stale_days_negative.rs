use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_stale_days_negative() {
    let args = vec!["--stale-days".to_string(), "-1".to_string()];
    let result = parse_diet_args(&args);
    assert!(result.is_err());
}
