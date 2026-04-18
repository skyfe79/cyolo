use super::super::*;
use std::path::PathBuf;

#[test]
fn test_display_profile_file_error_includes_path_and_message() {
    owo_colors::set_override(false);
    let err = CyoloError::ProfileFileError {
        path: PathBuf::from("/tmp/cyolo/profiles/broken.json"),
        message: "missing required field".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("/tmp/cyolo/profiles/broken.json"));
    assert!(msg.contains("missing required field"));
}
