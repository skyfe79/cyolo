use super::super::*;
use tempfile::TempDir;

/// Minimal JSON with only `profiles` should load with `default: None`
/// thanks to `#[serde(default)]`.
#[test]
fn test_load_in_minimal_profiles_only_json() {
    let tmp = TempDir::new().unwrap();
    ensure_dir_in(tmp.path()).unwrap();
    fs::write(config_path_in(tmp.path()), b"{\"profiles\": {}}").unwrap();

    let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
    assert!(loaded.default.is_none());
    assert!(loaded.profiles.is_empty());
}
