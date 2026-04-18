use super::super::*;
use super::common::*;

#[test]
fn test_parse_args_empty() {
    let result = parse_diet_args(&[]).unwrap();
    assert!(!result.apply);
    assert!(!result.force);
    assert!(result.stale_days.is_none());
    assert!(!result.cache);
    assert!(result.profile.is_none());
    assert!(!result.all);
}
