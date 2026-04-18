use super::super::*;

#[test]
fn test_unresolved_profile_omits_claude_config_dir() {
    let env = build_env(None);
    assert!(
        env.is_empty(),
        "expected empty env diff for unresolved profile, got: {:?}",
        env,
    );
    assert!(env.iter().all(|(k, _)| k != "CLAUDE_CONFIG_DIR"));
}
