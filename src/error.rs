use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CyoloError {
    #[error("cyolo: claude not found in PATH.\n  Install Claude Code: https://docs.anthropic.com/en/docs/claude-code")]
    ClaudeNotFound,

    #[error("cyolo: failed to execute claude at {path}: {source}")]
    ClaudeExecFailed {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("")]
    NonZeroExit(i32),

    #[error("cyolo: {0} is not yet implemented")]
    NotImplemented(String),
}
