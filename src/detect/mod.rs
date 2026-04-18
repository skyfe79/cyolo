use std::fs;
use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;
use serde::Deserialize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;
use crate::util::expand_tilde;

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
            eprintln!(
                "{} could not determine current directory: {e}",
                "warning:".yellow().bold()
            );
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
    if let Some((ref path, ref pf)) = found
        && pf.name.is_none()
        && let Some(ref dir) = pf.config_dir
    {
        return Ok(Some(ResolvedProfile {
            name: None,
            config_dir: expand_tilde(dir),
            source: path.display().to_string(),
        }));
    }

    // Name lookup or default fallback — needs config
    let config = CyoloConfig::load()?;
    resolve_with(&config, found)
}


#[cfg(test)]
mod tests;
