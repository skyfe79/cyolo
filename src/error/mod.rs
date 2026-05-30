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

    #[error("cyolo: failed to parse config at {path}: {source}")]
    ConfigParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("cyolo: {context}: {source}")]
    ConfigIoError {
        context: String,
        #[source]
        source: std::io::Error,
    },

    #[error("cyolo: profile '{name}' already exists")]
    ProfileAlreadyExists { name: String },

    #[error("cyolo: profile '{name}' not found. Run: cyolo profile add {name}")]
    ProfileNotFound { name: String },

    #[error("cyolo: invalid profile file {path}: {message}")]
    ProfileFileError { path: PathBuf, message: String },

    #[error(
        "cyolo: claude at {path} is not a symlinked native install — version switching needs the ~/.local/share/claude/versions layout.\n  (Install Claude Code with the native installer to enable `cyolo version` / `cyolo update`.)"
    )]
    NotNativeInstall { path: PathBuf },

    #[error("cyolo: version '{version}' is not installed under the versions directory")]
    VersionNotInstalled { version: String },

    #[error("cyolo: couldn't fetch upstream versions: {message}")]
    RemoteFetchFailed { message: String },
}


#[cfg(test)]
mod tests;
