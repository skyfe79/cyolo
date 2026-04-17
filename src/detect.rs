use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::CyoloError;

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
}
