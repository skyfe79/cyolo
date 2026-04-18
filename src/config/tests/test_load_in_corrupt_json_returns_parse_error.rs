use super::super::*;
use tempfile::TempDir;

#[test]
fn test_load_in_corrupt_json_returns_parse_error() {
    let tmp = TempDir::new().unwrap();
    ensure_dir_in(tmp.path()).unwrap();
    fs::write(config_path_in(tmp.path()), b"not json").unwrap();

    let err = CyoloConfig::load_in(tmp.path()).unwrap_err();
    match err {
        CyoloError::ConfigParseError { path, .. } => {
            assert_eq!(path, config_path_in(tmp.path()));
        }
        other => panic!("expected ConfigParseError, got {other:?}"),
    }
}
