use std::path::{Path, PathBuf};
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
        if let Some(url) = &profile.anthropic_base_url {
            env.push(("ANTHROPIC_BASE_URL".to_string(), url.clone()));
        }
        if let Some(key) = &profile.anthropic_api_key {
            env.push(("ANTHROPIC_API_KEY".to_string(), key.clone()));
        }
        if let Some(model) = &profile.anthropic_model {
            env.push(("ANTHROPIC_MODEL".to_string(), model.clone()));
        }
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

/// Spawn `claude` with `CLAUDE_CONFIG_DIR=<config_dir>` for interactive OAuth
/// login into a specific profile.
///
/// Omits `--dangerously-skip-permissions` so the inner Claude Code prompt can
/// run `/login` normally. Claude Code hashes `CLAUDE_CONFIG_DIR` into its
/// keychain service name (`Claude Code-credentials-<sha256[:8]>`), so each
/// profile's token lands in a distinct Keychain entry.
pub fn run_claude_login(config_dir: &Path) -> Result<(), CyoloError> {
    let claude = find_claude()?;
    let status = Command::new(&claude)
        .env("CLAUDE_CONFIG_DIR", config_dir)
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
mod tests;
