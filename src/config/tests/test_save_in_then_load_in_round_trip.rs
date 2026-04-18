use super::super::*;
use std::collections::BTreeMap;
use std::path::PathBuf;
use tempfile::TempDir;

fn sample_config() -> CyoloConfig {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "work".to_string(),
        Profile {
            name: "work".to_string(),
            config_dir: PathBuf::from("/tmp/.claude-work"),
        },
    );
    profiles.insert(
        "personal".to_string(),
        Profile {
            name: "personal".to_string(),
            config_dir: PathBuf::from("/tmp/.claude-personal"),
        },
    );
    CyoloConfig {
        default: Some("work".to_string()),
        profiles,
    }
}

#[test]
fn test_save_in_then_load_in_round_trip() {
    let tmp = TempDir::new().unwrap();
    let cfg = sample_config();
    cfg.save_in(tmp.path()).unwrap();

    let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
    assert_eq!(cfg, loaded);
}
