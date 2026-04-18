use super::super::*;
use crate::config::CyoloConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn test_resolve_with_config_dir() {
    let config = CyoloConfig {
        default: None,
        profiles: BTreeMap::new(),
    };
    let pf = ProfileFile {
        name: None,
        config_dir: Some("/custom/dir".into()),
    };
    let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

    let result = resolve_with(&config, found).unwrap().unwrap();
    assert!(result.name.is_none());
    assert_eq!(result.config_dir, PathBuf::from("/custom/dir"));
    assert_eq!(result.source, "/project/.claude-profile.json");
}
