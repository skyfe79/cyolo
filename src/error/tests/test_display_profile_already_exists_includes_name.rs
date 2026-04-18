use super::super::*;

#[test]
fn test_display_profile_already_exists_includes_name() {
    owo_colors::set_override(false);
    let err = CyoloError::ProfileAlreadyExists {
        name: "personal".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("personal"));
}
