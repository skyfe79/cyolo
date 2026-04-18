use super::super::*;
use std::path::PathBuf;

#[test]
fn test_display_config_parse_error_includes_path_and_source() {
    owo_colors::set_override(false);
    let json_err = serde_json::from_str::<u32>("notjson").unwrap_err();
    let err = CyoloError::ConfigParseError {
        path: PathBuf::from("/tmp/cyolo/config.json"),
        source: json_err,
    };
    let msg = format!("{err}");
    assert!(msg.contains("/tmp/cyolo/config.json"));
    assert!(
        msg.contains("line") || msg.contains("column") || msg.contains("expected"),
        "expected serde_json error substring in: {msg}"
    );
}
