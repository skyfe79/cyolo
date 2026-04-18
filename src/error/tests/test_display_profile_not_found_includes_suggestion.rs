use super::super::*;

#[test]
fn test_display_profile_not_found_includes_suggestion() {
    owo_colors::set_override(false);
    let err = CyoloError::ProfileNotFound {
        name: "work".into(),
    };
    let msg = format!("{err}");
    assert!(msg.contains("work"));
    assert!(msg.contains("Run: cyolo profile add"));
}
