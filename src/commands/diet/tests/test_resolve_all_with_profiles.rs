use super::super::*;
use super::common::*;

#[test]
fn test_resolve_all_with_profiles() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("main".to_string(), Profile {
        name: "main".to_string(),
        config_dir: PathBuf::from("/home/user/.claude"),
    });
    profiles.insert("work".to_string(), Profile {
        name: "work".to_string(),
        config_dir: PathBuf::from("/home/user/.claude-work"),
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: true,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    // BTreeMap is sorted by key, so "main" comes before "work"
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "main");
    assert_eq!(result[1].0, "work");
}
