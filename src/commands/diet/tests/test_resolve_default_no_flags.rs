use super::super::*;
use super::common::*;

#[test]
fn test_resolve_default_no_flags() {
    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: false,
    };

    let result = resolve_target_profiles(&options).unwrap();

    assert_eq!(result.len(), 1);
    let (display_name, path) = &result[0];
    // Default path should be ~/.claude
    assert!(path.ends_with(".claude"));
    // Display name should use tilde contraction
    assert!(display_name.starts_with("~"), "expected tilde path, got: {display_name}");
}
