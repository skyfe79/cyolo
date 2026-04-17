use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::CyoloError;

/// Locate the `claude` binary in PATH.
pub fn find_claude() -> Result<PathBuf, CyoloError> {
    which::which("claude").map_err(|_| CyoloError::ClaudeNotFound)
}

/// Run `claude --dangerously-skip-permissions <args...>`.
/// Inherits stdin/stdout/stderr for interactive use.
/// When `config_dir` is `Some`, sets `CLAUDE_CONFIG_DIR` on the child process.
pub fn run_claude(args: &[String], config_dir: Option<&Path>) -> Result<(), CyoloError> {
    let claude = find_claude()?;
    let mut cmd = Command::new(&claude);
    cmd.arg("--dangerously-skip-permissions").args(args);

    if let Some(dir) = config_dir {
        cmd.env("CLAUDE_CONFIG_DIR", dir);
    }

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
