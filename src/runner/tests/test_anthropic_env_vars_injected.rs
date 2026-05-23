use super::super::*;
use std::path::PathBuf;

fn profile_with_provider(base_url: &str, api_key: &str, model: &str) -> ResolvedProfile {
    ResolvedProfile {
        name: Some("deepseek".into()),
        config_dir: PathBuf::from("/tmp/.claude-deepseek"),
        source: "test".into(),
        anthropic_base_url: Some(base_url.into()),
        anthropic_api_key: Some(api_key.into()),
        anthropic_model: Some(model.into()),
    }
}

#[test]
fn test_all_three_anthropic_vars_are_injected() {
    let p = profile_with_provider("https://api.deepseek.com", "sk-test", "deepseek-chat");
    let env = build_env(Some(&p));

    let get = |key: &str| {
        env.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    };

    assert_eq!(get("CLAUDE_CONFIG_DIR"), Some("/tmp/.claude-deepseek"));
    assert_eq!(get("ANTHROPIC_BASE_URL"), Some("https://api.deepseek.com"));
    assert_eq!(get("ANTHROPIC_API_KEY"), Some("sk-test"));
    assert_eq!(get("ANTHROPIC_MODEL"), Some("deepseek-chat"));
    assert_eq!(env.len(), 4);
}

#[test]
fn test_partial_anthropic_vars_are_injected() {
    let p = ResolvedProfile {
        name: Some("partial".into()),
        config_dir: PathBuf::from("/tmp/.claude-partial"),
        source: "test".into(),
        anthropic_base_url: Some("https://api.deepseek.com".into()),
        anthropic_api_key: None,
        anthropic_model: Some("deepseek-chat".into()),
    };
    let env = build_env(Some(&p));

    assert_eq!(env.len(), 3);
    assert!(env.iter().any(|(k, _)| k == "ANTHROPIC_BASE_URL"));
    assert!(env.iter().any(|(k, _)| k == "ANTHROPIC_MODEL"));
    assert!(!env.iter().any(|(k, _)| k == "ANTHROPIC_API_KEY"));
}

#[test]
fn test_no_anthropic_vars_when_none() {
    let p = ResolvedProfile {
        name: Some("plain".into()),
        config_dir: PathBuf::from("/tmp/.claude-plain"),
        source: "test".into(),
        anthropic_base_url: None,
        anthropic_api_key: None,
        anthropic_model: None,
    };
    let env = build_env(Some(&p));

    assert_eq!(env.len(), 1);
    assert_eq!(env[0].0, "CLAUDE_CONFIG_DIR");
}
