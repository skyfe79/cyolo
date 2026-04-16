use std::path::PathBuf;
use std::process::Command;

use crate::error::CyoloError;

/// Locate the `claude` binary in PATH.
pub fn find_claude() -> Result<PathBuf, CyoloError> {
    which::which("claude").map_err(|_| CyoloError::ClaudeNotFound)
}

/// Run `claude --dangerously-skip-permissions <args...>`.
/// Inherits stdin/stdout/stderr for interactive use.
pub fn run_claude(args: &[String]) -> Result<(), CyoloError> {
    let claude = find_claude()?;
    let status = Command::new(&claude)
        .arg("--dangerously-skip-permissions")
        .args(args)
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
