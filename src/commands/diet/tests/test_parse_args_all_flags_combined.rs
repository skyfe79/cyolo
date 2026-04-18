use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_all_flags_combined() {
    let args = vec![
        "--apply".to_string(),
        "--force".to_string(),
        "--stale-days".to_string(),
        "30".to_string(),
        "--cache".to_string(),
        "--all".to_string(),
    ];
    let result = parse_diet_args(&args).unwrap();
    assert!(result.apply);
    assert!(result.force);
    assert_eq!(result.stale_days, Some(30));
    assert!(result.cache);
    assert!(result.profile.is_none());
    assert!(result.all);
}
