use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::CyoloError;

/// Global configuration for cyolo, stored at `~/.cyolo/config.json`.
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct CyoloConfig {
    /// Name of the default profile, if set.
    #[serde(default)]
    pub default: Option<String>,
    /// Registered profiles keyed by name. BTreeMap gives deterministic key ordering.
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

/// A single profile entry in the config.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
}

/// Return the cyolo config directory inside `home`: `{home}/.cyolo/`.
fn config_dir_in(home: &Path) -> PathBuf {
    home.join(".cyolo")
}

/// Return the cyolo config file path inside `home`: `{home}/.cyolo/config.json`.
fn config_path_in(home: &Path) -> PathBuf {
    config_dir_in(home).join("config.json")
}

/// Create `{home}/.cyolo/` with mode `0o700` if it does not already exist.
fn ensure_dir_in(home: &Path) -> Result<(), CyoloError> {
    use std::os::unix::fs::DirBuilderExt;

    let dir = config_dir_in(home);
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

/// Resolve the current user's home directory or return a `ConfigIoError`.
///
/// This is the single place that calls `dirs::home_dir()`. Keeping the call
/// isolated lets the `*_in` helpers stay hermetic for testing.
fn resolve_home() -> Result<PathBuf, CyoloError> {
    dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
        context: "could not determine home directory".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
    })
}

/// Return the cyolo config directory: `~/.cyolo/`.
#[allow(dead_code)]
pub fn config_dir() -> Result<PathBuf, CyoloError> {
    Ok(config_dir_in(&resolve_home()?))
}

/// Return the cyolo config file path: `~/.cyolo/config.json`.
#[allow(dead_code)]
pub fn config_path() -> Result<PathBuf, CyoloError> {
    Ok(config_path_in(&resolve_home()?))
}

/// Create `~/.cyolo/` with mode `0o700` if it does not already exist.
pub fn ensure_dir() -> Result<(), CyoloError> {
    ensure_dir_in(&resolve_home()?)
}

impl CyoloConfig {
    /// Load config from `{home}/.cyolo/config.json`.
    ///
    /// Returns a default empty config if the file does not exist.
    /// Returns `ConfigParseError` if the file exists but contains malformed JSON.
    fn load_in(home: &Path) -> Result<Self, CyoloError> {
        let path = config_path_in(home);

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

    /// Save config to `{home}/.cyolo/config.json` using atomic write.
    ///
    /// Writes to a temporary file in the same directory, calls `sync_all()`,
    /// then renames over the target to prevent corruption on crash.
    /// Sets the target file mode to `0o600` after rename.
    fn save_in(&self, home: &Path) -> Result<(), CyoloError> {
        use std::io::Write;
        use std::os::unix::fs::PermissionsExt;

        ensure_dir_in(home)?;

        let path = config_path_in(home);
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

        // Tighten the temp file permissions to 0o600 before rename so the
        // final file never lives even briefly with umask-derived perms.
        fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
            CyoloError::ConfigIoError {
                context: format!("failed to chmod temp file {}", tmp_path.display()),
                source: e,
            }
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

    /// Load config from `~/.cyolo/config.json`.
    ///
    /// Returns a default empty config if the file does not exist.
    /// Returns `ConfigParseError` if the file exists but contains malformed JSON.
    pub fn load() -> Result<Self, CyoloError> {
        Self::load_in(&resolve_home()?)
    }

    /// Save config to `~/.cyolo/config.json` using atomic write.
    pub fn save(&self) -> Result<(), CyoloError> {
        self.save_in(&resolve_home()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn sample_config() -> CyoloConfig {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "work".to_string(),
            Profile {
                name: "work".to_string(),
                config_dir: PathBuf::from("/tmp/.claude-work"),
            },
        );
        profiles.insert(
            "personal".to_string(),
            Profile {
                name: "personal".to_string(),
                config_dir: PathBuf::from("/tmp/.claude-personal"),
            },
        );
        CyoloConfig {
            default: Some("work".to_string()),
            profiles,
        }
    }

    #[test]
    fn test_config_dir_in_joins_cyolo() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(config_dir_in(tmp.path()), tmp.path().join(".cyolo"));
    }

    #[test]
    fn test_config_path_in_joins_config_json() {
        let tmp = TempDir::new().unwrap();
        assert_eq!(
            config_path_in(tmp.path()),
            tmp.path().join(".cyolo").join("config.json")
        );
    }

    #[test]
    fn test_save_in_then_load_in_round_trip() {
        let tmp = TempDir::new().unwrap();
        let cfg = sample_config();
        cfg.save_in(tmp.path()).unwrap();

        let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
        assert_eq!(cfg, loaded);
    }

    #[test]
    fn test_load_in_missing_file_returns_default() {
        // Current contract: load_in on a fresh dir where config.json does
        // not exist returns Ok(Self::default()) — an empty config.
        let tmp = TempDir::new().unwrap();
        let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
        assert_eq!(loaded, CyoloConfig::default());
        assert!(loaded.default.is_none());
        assert!(loaded.profiles.is_empty());
    }

    #[test]
    fn test_load_in_minimal_profiles_only_json() {
        // Minimal JSON with only `profiles` should load with default: None
        // thanks to #[serde(default)].
        let tmp = TempDir::new().unwrap();
        ensure_dir_in(tmp.path()).unwrap();
        fs::write(config_path_in(tmp.path()), b"{\"profiles\": {}}").unwrap();

        let loaded = CyoloConfig::load_in(tmp.path()).unwrap();
        assert!(loaded.default.is_none());
        assert!(loaded.profiles.is_empty());
    }

    #[test]
    fn test_load_in_corrupt_json_returns_parse_error() {
        let tmp = TempDir::new().unwrap();
        ensure_dir_in(tmp.path()).unwrap();
        fs::write(config_path_in(tmp.path()), b"not json").unwrap();

        let err = CyoloConfig::load_in(tmp.path()).unwrap_err();
        match err {
            CyoloError::ConfigParseError { path, .. } => {
                assert_eq!(path, config_path_in(tmp.path()));
            }
            other => panic!("expected ConfigParseError, got {other:?}"),
        }
    }

    #[cfg(unix)]
    #[test]
    fn test_save_in_permissions_dir_0700_file_0600() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = TempDir::new().unwrap();
        sample_config().save_in(tmp.path()).unwrap();

        let dir_mode = fs::metadata(config_dir_in(tmp.path()))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            dir_mode & 0o777,
            0o700,
            "config dir should be 0o700, got {:o}",
            dir_mode & 0o777
        );

        let file_mode = fs::metadata(config_path_in(tmp.path()))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            file_mode & 0o777,
            0o600,
            "config file should be 0o600, got {:o}",
            file_mode & 0o777
        );
    }

    #[test]
    fn test_ensure_dir_in_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        ensure_dir_in(tmp.path()).unwrap();
        assert!(config_dir_in(tmp.path()).is_dir());
        // Second call must not error when the directory already exists.
        ensure_dir_in(tmp.path()).unwrap();
        assert!(config_dir_in(tmp.path()).is_dir());
    }
}
