use super::super::*;
use tempfile::TempDir;

/// Current contract: `load_in` on a fresh dir where `config.json` does
/// not exist returns `Ok(Self::default())` — an empty config.
#[test]
fn test_load_in_missing_file_returns_default() {
    let tmp = TempDir::new().unwrap();
    let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
    assert_eq!(loaded, CyoloConfig::default());
    assert!(loaded.default.is_none());
    assert!(loaded.profiles.is_empty());
}
