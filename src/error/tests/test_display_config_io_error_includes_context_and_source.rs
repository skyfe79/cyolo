use super::super::*;

#[test]
fn test_display_config_io_error_includes_context_and_source() {
    owo_colors::set_override(false);
    let io_err = std::io::Error::other("boom");
    let err = CyoloError::ConfigIoError {
        context: "reading config".into(),
        source: io_err,
    };
    let msg = format!("{err}");
    assert!(msg.contains("reading config"));
    assert!(msg.contains("boom"));
}
