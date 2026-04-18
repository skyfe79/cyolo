use super::super::*;
use crate::config::CyoloConfig;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[test]
fn test_resolve_with_unregistered_name() {
    let config = CyoloConfig {
        default: None,
        profiles: BTreeMap::new(),
    };
    let pf = ProfileFile {
        name: Some("unknown".into()),
        config_dir: None,
    };
    let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

    let err = resolve_with(&config, found).unwrap_err();
    match &err {
        CyoloError::ProfileNotFound { name } => assert_eq!(name, "unknown"),
        other => panic!("expected ProfileNotFound, got: {other:?}"),
    }
}
