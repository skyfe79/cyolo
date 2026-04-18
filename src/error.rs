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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Once;

    static INIT_COLORS: Once = Once::new();
    fn setup() {
        INIT_COLORS.call_once(|| owo_colors::set_override(false));
    }

    #[test]
    fn display_config_parse_error_includes_path_and_source() {
        setup();
        let json_err = serde_json::from_str::<u32>("notjson").unwrap_err();
        let err = CyoloError::ConfigParseError {
            path: PathBuf::from("/tmp/cyolo/config.json"),
            source: json_err,
        };
        let msg = format!("{err}");
        assert!(msg.contains("/tmp/cyolo/config.json"));
        assert!(
            msg.contains("line") || msg.contains("column") || msg.contains("expected"),
            "expected serde_json error substring in: {msg}"
        );
    }

    #[test]
    fn display_config_io_error_includes_context_and_source() {
        setup();
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "boom");
        let err = CyoloError::ConfigIoError {
            context: "reading config".into(),
            source: io_err,
        };
        let msg = format!("{err}");
        assert!(msg.contains("reading config"));
        assert!(msg.contains("boom"));
    }

    #[test]
    fn display_profile_not_found_includes_suggestion() {
        setup();
        let err = CyoloError::ProfileNotFound {
            name: "work".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("work"));
        assert!(msg.contains("Run: cyolo profile add"));
    }

    #[test]
    fn display_profile_already_exists_includes_name() {
        setup();
        let err = CyoloError::ProfileAlreadyExists {
            name: "personal".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("personal"));
    }

    #[test]
    fn display_profile_file_error_includes_path_and_message() {
        setup();
        let err = CyoloError::ProfileFileError {
            path: PathBuf::from("/tmp/cyolo/profiles/broken.json"),
            message: "missing required field".into(),
        };
        let msg = format!("{err}");
        assert!(msg.contains("/tmp/cyolo/profiles/broken.json"));
        assert!(msg.contains("missing required field"));
    }
}
