use super::super::*;
use super::common::*;

#[test]
fn test_resolve_profile_found() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("work".to_string(), Profile {
        name: "work".to_string(),
        config_dir: PathBuf::from("/home/user/.claude-work"),
        anthropic_base_url: None,
        anthropic_api_key: None,
        anthropic_model: None,
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: Some("work".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "work");
    assert_eq!(result[0].1, PathBuf::from("/home/user/.claude-work"));
}
