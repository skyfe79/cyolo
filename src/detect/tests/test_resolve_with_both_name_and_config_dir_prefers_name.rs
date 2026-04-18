use super::super::*;
use crate::config::{CyoloConfig, Profile};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Product PRD §4.1: when `.claude-profile.json` carries both `name` and
/// `config_dir`, `name` wins and the inline `config_dir` is ignored.
#[test]
fn test_resolve_with_both_name_and_config_dir_prefers_name() {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "work".to_string(),
        Profile {
            name: "work".into(),
            config_dir: PathBuf::from("/home/user/.claude-work"),
        },
    );
    let config = CyoloConfig {
        default: None,
        profiles,
    };
    let pf = ProfileFile {
        name: Some("work".into()),
        config_dir: Some("/inline/should/be/ignored".into()),
    };
    let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

    let result = resolve_with(&config, found).unwrap().unwrap();
    assert_eq!(result.name.as_deref(), Some("work"));
    assert_eq!(result.config_dir, PathBuf::from("/home/user/.claude-work"));
}
