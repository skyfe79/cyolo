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

/// Contract: `build_env` is pure and does NOT expand tilde. Tilde expansion
/// happens earlier in `detect::resolve_with`, before `ResolvedProfile` is
/// built. A manually-constructed profile with `~/...` must survive untouched.
#[test]
fn test_tilde_prefixed_config_dir_is_passed_through_verbatim() {
    let p = profile("~/my-claude-config");
    let env = build_env(Some(&p));
    assert_eq!(env.len(), 1);
    let (k, v) = &env[0];
    assert_eq!(k, "CLAUDE_CONFIG_DIR");
    assert_eq!(
        v, "~/my-claude-config",
        "build_env must pass config_dir through verbatim (no tilde expansion)"
    );
}
