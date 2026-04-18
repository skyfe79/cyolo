use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_stale_days() {
    let args = vec!["--stale-days".to_string(), "90".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert_eq!(result.stale_days, Some(90));
}
