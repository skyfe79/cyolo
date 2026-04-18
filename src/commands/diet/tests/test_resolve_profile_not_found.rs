use super::super::*;
use super::common::*;

#[test]
fn test_resolve_profile_not_found() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
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
        profile: Some("nonexistent".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found"),
        "expected 'not found' in error, got: {err_msg}"
    );
}
