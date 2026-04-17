use std::path::PathBuf;
use std::process::Command;

use crate::detect::ResolvedProfile;
use crate::error::CyoloError;

/// Locate the `claude` binary in PATH.
pub fn find_claude() -> Result<PathBuf, CyoloError> {
    which::which("claude").map_err(|_| CyoloError::ClaudeNotFound)
}

/// Build the environment-variable diff applied to the child `claude` process.
///
/// Pure function — no I/O, no tilde expansion (the profile is assumed to already
/// hold a canonical path; `detect::resolve_with` expands tilde before constructing
/// the `ResolvedProfile`). Returning a `Vec` of `(key, value)` pairs lets the
/// caller feed it directly into `Command::envs`.
///
/// Contract:
/// - `Some(profile)` → single entry `("CLAUDE_CONFIG_DIR", profile.config_dir)`
/// - `None`          → empty vec (child inherits the parent's `CLAUDE_CONFIG_DIR`,
///   if any, unchanged — matches v1 PRD §3.1 point 3).
pub(crate) fn build_env(profile: Option<&ResolvedProfile>) -> Vec<(String, String)> {
    let mut env = Vec::new();
    if let Some(profile) = profile {
        env.push((
            "CLAUDE_CONFIG_DIR".to_string(),
            profile.config_dir.to_string_lossy().into_owned(),
        ));
    }
    env
}

/// Run `claude --dangerously-skip-permissions <args...>`.
/// Inherits stdin/stdout/stderr for interactive use.
/// When `resolved` is `Some`, sets `CLAUDE_CONFIG_DIR` on the child process.
pub fn run_claude(args: &[String], resolved: Option<&ResolvedProfile>) -> Result<(), CyoloError> {
    let claude = find_claude()?;
    let mut cmd = Command::new(&claude);
    cmd.arg("--dangerously-skip-permissions").args(args);
    cmd.envs(build_env(resolved));

    let status = cmd
        .status()
        .map_err(|e| CyoloError::ClaudeExecFailed {
            path: claude.clone(),
            source: e,
        })?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(CyoloError::NonZeroExit(code)),
        None => Err(CyoloError::NonZeroExit(1)), // killed by signal
    }
}

/// Run `claude update`.
/// No `--dangerously-skip-permissions` flag.
pub fn run_update() -> Result<(), CyoloError> {
    let claude = find_claude()?;
    let status = Command::new(&claude)
        .arg("update")
        .status()
        .map_err(|e| CyoloError::ClaudeExecFailed {
            path: claude.clone(),
            source: e,
        })?;

    match status.code() {
        Some(0) => Ok(()),
        Some(code) => Err(CyoloError::NonZeroExit(code)),
        None => Err(CyoloError::NonZeroExit(1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn profile(dir: &str) -> ResolvedProfile {
        ResolvedProfile {
            name: Some("test".into()),
            config_dir: PathBuf::from(dir),
            source: "test".into(),
        }
    }

    #[test]
    fn resolved_profile_sets_claude_config_dir() {
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

    #[test]
    fn unresolved_profile_omits_claude_config_dir() {
        let env = build_env(None);
        assert!(
            env.is_empty(),
            "expected empty env diff for unresolved profile, got: {:?}",
            env,
        );
        assert!(env.iter().all(|(k, _)| k != "CLAUDE_CONFIG_DIR"));
    }

    #[test]
    fn tilde_prefixed_config_dir_is_passed_through_verbatim() {
        // Contract: build_env is pure and does NOT expand tilde. Tilde expansion
        // happens earlier in detect::resolve_with, before ResolvedProfile is built.
        // A manually-constructed profile with "~/..." must survive untouched.
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
}
