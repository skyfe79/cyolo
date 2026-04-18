use super::super::*;
use crate::config::CyoloConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn test_resolve_with_config_dir_tilde() {
    let config = CyoloConfig {
        default: None,
        profiles: BTreeMap::new(),
    };
    let pf = ProfileFile {
        name: None,
        config_dir: Some("~/my-claude-config".into()),
    };
    let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

    let result = resolve_with(&config, found).unwrap().unwrap();
    assert!(result.name.is_none());
    // Tilde must be expanded to an absolute path.
    assert!(
        !result.config_dir.starts_with("~"),
        "tilde was not expanded: {:?}",
        result.config_dir
    );
    assert!(
        result.config_dir.ends_with("my-claude-config"),
        "unexpected path: {:?}",
        result.config_dir
    );
}
