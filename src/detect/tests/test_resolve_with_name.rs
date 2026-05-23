use super::super::*;
use crate::config::{CyoloConfig, Profile};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn make_config(profiles: &[(&str, &str)], default: Option<&str>) -> CyoloConfig {
    let mut map = BTreeMap::new();
    for (name, dir) in profiles {
        map.insert(
            name.to_string(),
            Profile {
                name: name.to_string(),
                config_dir: PathBuf::from(dir),
                anthropic_base_url: None,
                anthropic_api_key: None,
                anthropic_model: None,
            },
        );
    }
    CyoloConfig {
        default: default.map(|s| s.to_string()),
        profiles: map,
    }
}

#[test]
fn test_resolve_with_name() {
    let config = make_config(&[("work", "/home/user/.claude-work")], None);
    let pf = ProfileFile {
        name: Some("work".into()),
        config_dir: None,
    };
    let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

    let result = resolve_with(&config, found).unwrap().unwrap();
    assert_eq!(result.name.as_deref(), Some("work"));
    assert_eq!(result.config_dir, PathBuf::from("/home/user/.claude-work"));
    assert_eq!(result.source, "/project/.claude-profile.json");
}
