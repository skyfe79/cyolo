use super::super::*;
use crate::config::{CyoloConfig, Profile};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn test_resolve_default_fallback() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "main".to_string(),
        Profile {
            name: "main".into(),
            config_dir: PathBuf::from("/home/user/.claude-main"),
        },
    );
    let config = CyoloConfig {
        default: Some("main".into()),
        profiles,
    };
    let result = resolve_with(&config, None).unwrap().unwrap();
    assert_eq!(result.name.as_deref(), Some("main"));
    assert_eq!(result.config_dir, PathBuf::from("/home/user/.claude-main"));
    assert_eq!(result.source, "default");
}
