use super::super::*;
use std::path::PathBuf;

fn profile(dir: &str) -> ResolvedProfile {
    ResolvedProfile {
        name: Some("test".into()),
        config_dir: PathBuf::from(dir),
        source: "test".into(),
        anthropic_base_url: None,
        anthropic_api_key: None,
        anthropic_model: None,
    }
}

#[test]
fn test_resolved_profile_sets_claude_config_dir() {
    let p = profile("/tmp/fake-profile");
    let env = build_env(Some(&p));
    assert_eq!(env.len(), 1);
    assert!(
        env.iter()
            .any(|(k, v)| k == "CLAUDE_CONFIG_DIR" && v == "/tmp/fake-profile"),
        "expected CLAUDE_CONFIG_DIR=/tmp/fake-profile, got: {:?}",
        env,
    );
}
