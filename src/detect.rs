use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;
use crate::profile::expand_tilde;

/// The `.claude-profile.json` file schema.
///
/// At least one of `name` or `config_dir` must be present.
/// If `name` is present, it will be resolved via the global profile registry.
/// If only `config_dir` is present, it will be used directly (with tilde expansion).
#[derive(Debug, Deserialize)]
pub struct ProfileFile {
    pub name: Option<String>,
    pub config_dir: Option<String>,
}

const PROFILE_FILENAME: &str = ".claude-profile.json";

impl ProfileFile {
    /// Read and parse a `.claude-profile.json` file from disk.
    ///
    /// Returns `ConfigParseError` for malformed JSON and `ProfileFileError`
    /// when the file has valid JSON but no recognized fields.
    pub fn from_file(path: &Path) -> Result<Self, CyoloError> {
        let bytes = fs::read(path).map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to read {}", path.display()),
            source: e,
        })?;
        parse(&bytes, path)
    }
}

/// Parse raw bytes as a `ProfileFile`, validating that at least one
/// recognized field is present.
fn parse(bytes: &[u8], path: &Path) -> Result<ProfileFile, CyoloError> {
    let pf: ProfileFile =
        serde_json::from_slice(bytes).map_err(|e| CyoloError::ConfigParseError {
            path: path.to_path_buf(),
            source: e,
        })?;

    if pf.name.is_none() && pf.config_dir.is_none() {
        return Err(CyoloError::ProfileFileError {
            path: path.to_path_buf(),
            message: "expected 'name' or 'config_dir' field".into(),
        });
    }

    Ok(pf)
}

/// Walk up from `start`, stopping after checking `boundary` (inclusive).
///
/// When `boundary` is `None`, walks all the way to the filesystem root.
fn walk_up_search(
    start: &Path,
    boundary: Option<&Path>,
) -> Result<Option<(PathBuf, ProfileFile)>, CyoloError> {
    for ancestor in start.ancestors() {
        let candidate = ancestor.join(PROFILE_FILENAME);
        if candidate.exists() {
            let pf = ProfileFile::from_file(&candidate)?;
            return Ok(Some((candidate, pf)));
        }
        if boundary.is_some_and(|b| ancestor == b) {
            break;
        }
    }
    Ok(None)
}

/// Walk up the directory tree from `start` looking for `.claude-profile.json`.
///
/// Returns `Some((path, profile_file))` if found, `None` otherwise.
pub fn find_profile_file_from(
    start: &Path,
) -> Result<Option<(PathBuf, ProfileFile)>, CyoloError> {
    walk_up_search(start, None)
}

/// Walk up from the current working directory looking for `.claude-profile.json`.
///
/// If `current_dir()` fails, prints a warning to stderr and returns `Ok(None)`.
pub fn find_profile_file() -> Result<Option<(PathBuf, ProfileFile)>, CyoloError> {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("cyolo: warning: could not determine current directory: {e}");
            return Ok(None);
        }
    };
    find_profile_file_from(&cwd)
}

/// A resolved profile ready for use by the runner.
#[derive(Debug)]
#[allow(dead_code)]
pub struct ResolvedProfile {
    /// Profile name, if resolved via a named profile.
    pub name: Option<String>,
    /// Absolute path to the profile's config directory.
    pub config_dir: PathBuf,
    /// Where this resolution came from: the walk-up file path or `"default"`.
    pub source: String,
}

/// Resolve a profile from a walk-up result and config, without disk I/O.
///
/// This is the pure-logic core, separated for testability.
fn resolve_with(
    config: &CyoloConfig,
    found: Option<(PathBuf, ProfileFile)>,
) -> Result<Option<ResolvedProfile>, CyoloError> {
    if let Some((path, pf)) = found {
        let source = path.display().to_string();

        // name variant takes priority
        if let Some(ref name) = pf.name {
            let profile = config
                .profiles
                .get(name)
                .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;
            return Ok(Some(ResolvedProfile {
                name: Some(name.clone()),
                config_dir: profile.config_dir.clone(),
                source,
            }));
        }

        // config_dir variant
        if let Some(ref dir) = pf.config_dir {
            return Ok(Some(ResolvedProfile {
                name: None,
                config_dir: expand_tilde(dir),
                source,
            }));
        }
    }

    // No walk-up file — check default
    if let Some(ref default_name) = config.default {
        let profile = config
            .profiles
            .get(default_name)
            .ok_or_else(|| CyoloError::ProfileNotFound {
                name: default_name.clone(),
            })?;
        return Ok(Some(ResolvedProfile {
            name: Some(default_name.clone()),
            config_dir: profile.config_dir.clone(),
            source: "default".into(),
        }));
    }

    Ok(None)
}

/// Resolve the active profile using the full priority chain.
///
/// 1. Walk up from cwd looking for `.claude-profile.json`
/// 2. Fall back to the default profile in `~/.cyolo/config.json`
/// 3. Return `None` if neither is available
///
/// The global config (`~/.cyolo/config.json`) is loaded lazily — only when
/// a named profile lookup or default fallback is needed. A walk-up file
/// with a direct `config_dir` never touches the global config.
pub fn resolve_profile() -> Result<Option<ResolvedProfile>, CyoloError> {
    let found = find_profile_file()?;

    // config_dir-only walk-up doesn't need the global config
    if let Some((ref path, ref pf)) = found {
        if pf.name.is_none() {
            if let Some(ref dir) = pf.config_dir {
                return Ok(Some(ResolvedProfile {
                    name: None,
                    config_dir: expand_tilde(dir),
                    source: path.display().to_string(),
                }));
            }
        }
    }

    // Name lookup or default fallback — needs config
    let config = CyoloConfig::load()?;
    resolve_with(&config, found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // --- Parsing tests (no disk I/O) ---

    #[test]
    fn test_parse_name_only() {
        let json = br#"{"name": "work"}"#;
        let pf = parse(json, Path::new("test.json")).unwrap();
        assert_eq!(pf.name.as_deref(), Some("work"));
        assert!(pf.config_dir.is_none());
    }

    #[test]
    fn test_parse_config_dir_only() {
        let json = br#"{"config_dir": "~/.claude-custom"}"#;
        let pf = parse(json, Path::new("test.json")).unwrap();
        assert!(pf.name.is_none());
        assert_eq!(pf.config_dir.as_deref(), Some("~/.claude-custom"));
    }

    #[test]
    fn test_parse_both() {
        let json = br#"{"name": "x", "config_dir": "y"}"#;
        let pf = parse(json, Path::new("test.json")).unwrap();
        assert_eq!(pf.name.as_deref(), Some("x"));
        assert_eq!(pf.config_dir.as_deref(), Some("y"));
    }

    #[test]
    fn test_parse_empty_object() {
        let json = br#"{}"#;
        let err = parse(json, Path::new("test.json")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expected 'name' or 'config_dir' field"), "got: {msg}");
    }

    #[test]
    fn test_parse_malformed_json() {
        let json = b"not json";
        let err = parse(json, Path::new("test.json")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("failed to parse config"), "got: {msg}");
    }

    // --- Walk-up tests (temp directory with boundary for determinism) ---

    #[test]
    fn test_find_in_start_dir() {
        let tmp = TempDir::new().unwrap();
        let profile_path = tmp.path().join(PROFILE_FILENAME);
        fs::write(&profile_path, r#"{"name": "local"}"#).unwrap();

        let result = walk_up_search(tmp.path(), Some(tmp.path())).unwrap();
        assert!(result.is_some());
        let (path, pf) = result.unwrap();
        assert_eq!(path, profile_path);
        assert_eq!(pf.name.as_deref(), Some("local"));
    }

    #[test]
    fn test_find_in_ancestor() {
        let tmp = TempDir::new().unwrap();
        let profile_path = tmp.path().join(PROFILE_FILENAME);
        fs::write(&profile_path, r#"{"config_dir": "/custom"}"#).unwrap();

        let child = tmp.path().join("sub").join("deep");
        fs::create_dir_all(&child).unwrap();

        let result = walk_up_search(&child, Some(tmp.path())).unwrap();
        assert!(result.is_some());
        let (path, pf) = result.unwrap();
        assert_eq!(path, profile_path);
        assert_eq!(pf.config_dir.as_deref(), Some("/custom"));
    }

    #[test]
    fn test_find_returns_none_when_not_found() {
        let tmp = TempDir::new().unwrap();
        let child = tmp.path().join("a").join("b");
        fs::create_dir_all(&child).unwrap();

        // Bounded walk-up: guaranteed no .claude-profile.json within tmp
        let result = walk_up_search(&child, Some(tmp.path())).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_unknown_fields_only() {
        let json = br#"{"other": "field"}"#;
        let err = parse(json, Path::new("test.json")).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("expected 'name' or 'config_dir' field"), "got: {msg}");
    }

    // --- resolve_with tests (pure logic, no disk I/O) ---

    use crate::config::{CyoloConfig, Profile};
    use std::collections::BTreeMap;

    fn make_config(profiles: &[(&str, &str)], default: Option<&str>) -> CyoloConfig {
        let mut map = BTreeMap::new();
        for (name, dir) in profiles {
            map.insert(
                name.to_string(),
                Profile {
                    name: name.to_string(),
                    config_dir: PathBuf::from(dir),
                },
            );
        }
        CyoloConfig {
            default: default.map(|s| s.to_string()),
            profiles: map,
        }
    }

    #[test]
    fn test_resolve_with_name() {
        let config = make_config(&[("work", "/home/user/.claude-work")], None);
        let pf = ProfileFile {
            name: Some("work".into()),
            config_dir: None,
        };
        let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

        let result = resolve_with(&config, found).unwrap().unwrap();
        assert_eq!(result.name.as_deref(), Some("work"));
        assert_eq!(result.config_dir, PathBuf::from("/home/user/.claude-work"));
        assert_eq!(result.source, "/project/.claude-profile.json");
    }

    #[test]
    fn test_resolve_with_unregistered_name() {
        let config = make_config(&[], None);
        let pf = ProfileFile {
            name: Some("unknown".into()),
            config_dir: None,
        };
        let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

        let err = resolve_with(&config, found).unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("unknown"), "got: {msg}");
    }

    #[test]
    fn test_resolve_with_config_dir() {
        let config = make_config(&[], None);
        let pf = ProfileFile {
            name: None,
            config_dir: Some("/custom/dir".into()),
        };
        let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

        let result = resolve_with(&config, found).unwrap().unwrap();
        assert!(result.name.is_none());
        assert_eq!(result.config_dir, PathBuf::from("/custom/dir"));
        assert_eq!(result.source, "/project/.claude-profile.json");
    }

    #[test]
    fn test_resolve_default_fallback() {
        let config = make_config(&[("main", "/home/user/.claude-main")], Some("main"));
        let result = resolve_with(&config, None).unwrap().unwrap();
        assert_eq!(result.name.as_deref(), Some("main"));
        assert_eq!(result.config_dir, PathBuf::from("/home/user/.claude-main"));
        assert_eq!(result.source, "default");
    }

    #[test]
    fn test_resolve_with_config_dir_tilde() {
        let config = make_config(&[], None);
        let pf = ProfileFile {
            name: None,
            config_dir: Some("~/my-claude-config".into()),
        };
        let found = Some((PathBuf::from("/project/.claude-profile.json"), pf));

        let result = resolve_with(&config, found).unwrap().unwrap();
        assert!(result.name.is_none());
        // Tilde must be expanded to an absolute path
        assert!(
            !result.config_dir.starts_with("~"),
            "tilde was not expanded: {:?}",
            result.config_dir
        );
        assert!(
            result.config_dir.ends_with("my-claude-config"),
            "unexpected path: {:?}",
            result.config_dir
        );
    }

    #[test]
    fn test_resolve_none() {
        let config = make_config(&[], None);
        let result = resolve_with(&config, None).unwrap();
        assert!(result.is_none());
    }
}
