use super::super::*;
use std::collections::BTreeMap;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn sample_config() -> CyoloConfig {
    let mut profiles = BTreeMap::new();
    profiles.insert(
        "work".to_string(),
        Profile {
            name: "work".to_string(),
            config_dir: PathBuf::from("/tmp/.claude-work"),
            anthropic_base_url: None,
            anthropic_api_key: None,
            anthropic_model: None,
        },
    );
    CyoloConfig {
        default: Some("work".to_string()),
        profiles,
    }
}

#[cfg(unix)]
#[test]
fn test_save_in_permissions_dir_0700_file_0600() {
    let tmp = TempDir::new().unwrap();
    sample_config().save_in(tmp.path()).unwrap();

    let dir_mode = fs::metadata(config_dir_in(tmp.path()))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        dir_mode & 0o777,
        0o700,
        "config dir should be 0o700, got {:o}",
        dir_mode & 0o777
    );

    let file_mode = fs::metadata(config_path_in(tmp.path()))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(
        file_mode & 0o777,
        0o600,
        "config file should be 0o600, got {:o}",
        file_mode & 0o777
    );
}
