use super::super::*;
use super::common::*;

#[test]
fn test_resolve_all_no_profiles() {
    use std::collections::BTreeMap;
    let cfg = CyoloConfig { default: None, profiles: BTreeMap::new() };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: true,
    };

    let result = resolve_profiles_from_config(&options, cfg);
    assert!(result.is_err());
}
