use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::CyoloError;

/// Global configuration for cyolo, stored at `~/.cyolo/config.json`.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CyoloConfig {
    /// Name of the default profile, if set.
    #[serde(default)]
    pub default: Option<String>,
    /// Registered profiles keyed by name. BTreeMap gives deterministic key ordering.
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

/// A single profile entry in the config.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
}

/// Return the cyolo config directory: `~/.cyolo/`.
pub fn config_dir() -> Result<PathBuf, CyoloError> {
    let home = dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
        context: "could not determine home directory".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
    })?;
    Ok(home.join(".cyolo"))
}

/// Return the cyolo config file path: `~/.cyolo/config.json`.
pub fn config_path() -> Result<PathBuf, CyoloError> {
    Ok(config_dir()?.join("config.json"))
}

/// Create `~/.cyolo/` with mode `0o700` if it does not already exist.
pub fn ensure_dir() -> Result<(), CyoloError> {
    use std::os::unix::fs::DirBuilderExt;

    let dir = config_dir()?;
    if !dir.exists() {
        fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&dir)
            .map_err(|e| CyoloError::ConfigIoError {
                context: format!("failed to create directory {}", dir.display()),
                source: e,
            })?;
    }
    Ok(())
}

impl CyoloConfig {
    /// Load config from `~/.cyolo/config.json`.
    ///
    /// Returns a default empty config if the file does not exist.
    /// Returns `ConfigParseError` if the file exists but contains malformed JSON.
    pub fn load() -> Result<Self, CyoloError> {
        let path = config_path()?;

        match fs::read(&path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|e| CyoloError::ConfigParseError {
                    path,
                    source: e,
                })
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(CyoloError::ConfigIoError {
                context: format!("failed to read config at {}", path.display()),
                source: e,
            }),
        }
    }

    /// Save config to `~/.cyolo/config.json` using atomic write.
    ///
    /// Writes to a temporary file in the same directory, calls `sync_all()`,
    /// then renames over the target to prevent corruption on crash.
    pub fn save(&self) -> Result<(), CyoloError> {
        use std::io::Write;

        ensure_dir()?;

        let path = config_path()?;
        let tmp_path = path.with_extension("json.tmp");

        let json = serde_json::to_string_pretty(self).map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to serialize config for {}", path.display()),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;

        let mut file =
            fs::File::create(&tmp_path).map_err(|e| CyoloError::ConfigIoError {
                context: format!("failed to create temp file {}", tmp_path.display()),
                source: e,
            })?;

        file.write_all(json.as_bytes())
            .map_err(|e| CyoloError::ConfigIoError {
                context: format!("failed to write temp file {}", tmp_path.display()),
                source: e,
            })?;

        // Append trailing newline for POSIX compliance.
        file.write_all(b"\n")
            .map_err(|e| CyoloError::ConfigIoError {
                context: format!("failed to write temp file {}", tmp_path.display()),
                source: e,
            })?;

        file.sync_all().map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to sync temp file {}", tmp_path.display()),
            source: e,
        })?;

        fs::rename(&tmp_path, &path).map_err(|e| CyoloError::ConfigIoError {
            context: format!(
                "failed to rename {} to {}",
                tmp_path.display(),
                path.display()
            ),
            source: e,
        })?;

        Ok(())
    }
}
