use super::super::*;
use super::common::*;

#[test]
fn test_resolve_profile_tilde_expansion() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("custom".to_string(), Profile {
        name: "custom".to_string(),
        config_dir: PathBuf::from("~/.claude-custom"),
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
        profile: Some("custom".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    assert_eq!(result.len(), 1);
    // Should NOT start with ~ (tilde should be expanded)
    assert!(
        !result[0].1.to_string_lossy().starts_with('~'),
        "expected expanded path, got: {}",
        result[0].1.display()
    );
}
