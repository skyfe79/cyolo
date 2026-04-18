use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_profile() {
    let args = vec!["--profile".to_string(), "work".to_string()];
    let result = parse_diet_args(&args).unwrap();
    assert_eq!(result.profile, Some("work".to_string()));
}
