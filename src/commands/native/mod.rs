//! Native-install version layout discovery + atomic version switching.
//!
//! The Claude Code native installer lays out the launcher as a symlink into
//! a per-version binary directory:
//!
//! ```text
//!   ~/.local/bin/claude                         (symlink)  ->
//!   ~/.local/share/claude/versions/<version>    (executable)
//! ```
//!
//! `cyolo version` reads that layout and `cyolo update <version>` repoints
//! the launcher symlink. Both go through the [`discover`] +
//! [`installed_versions_in`] + [`switch_in`] helpers here so the two verbs
//! agree on exactly what "installed" and "current" mean.
//!
//! Following the rest of the codebase, the filesystem-touching helpers take
//! their base path as an argument (`*_in`) so tests can drive them against a
//! tempdir without a real install.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CyoloError;

/// A discovered native install: where the launcher symlink lives, where the
/// per-version binaries live, and which version is currently active.
#[derive(Debug, Clone, PartialEq)]
pub struct Install {
    /// The launcher symlink (e.g. `~/.local/bin/claude`) that we repoint.
    pub bin_link: PathBuf,
    /// Directory holding the version binaries (e.g. `~/.local/share/claude/versions`).
    pub versions_dir: PathBuf,
    /// The currently-linked version name, derived from the symlink target's
    /// file name. `None` only if the target somehow has no file name.
    pub current: Option<String>,
}

/// Locate the active install by resolving `claude` in `PATH`.
///
/// `which::which` returns the launcher path as found in `PATH` (the symlink,
/// not a canonicalized real path), which is exactly what [`discover_from`]
/// needs to read the version target.
pub fn discover() -> Result<Install, CyoloError> {
    let bin = crate::runner::find_claude()?;
    discover_from(&bin)
}

/// Discover the install layout from a specific launcher path.
///
/// The launcher MUST be a symlink into a versions directory — that's the
/// shape the native installer produces and the only shape we can safely
/// repoint. A non-symlink (npm/global install, a hand-copied binary) yields
/// [`CyoloError::NotNativeInstall`] rather than us guessing.
pub fn discover_from(bin_link: &Path) -> Result<Install, CyoloError> {
    let raw_target = fs::read_link(bin_link).map_err(|_| CyoloError::NotNativeInstall {
        path: bin_link.to_path_buf(),
    })?;
    // Resolve a relative link target against the launcher's directory so
    // `versions_dir` is always absolute, matching the install convention.
    let target = if raw_target.is_absolute() {
        raw_target
    } else {
        bin_link
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join(raw_target)
    };
    let versions_dir = target
        .parent()
        .ok_or_else(|| CyoloError::NotNativeInstall {
            path: bin_link.to_path_buf(),
        })?
        .to_path_buf();
    let current = target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned());
    Ok(Install {
        bin_link: bin_link.to_path_buf(),
        versions_dir,
        current,
    })
}

/// List installed version names — the entries directly under `versions_dir`,
/// newest first. Dotfile entries are skipped so a stray `.DS_Store` or our own
/// temp symlink never shows up as a "version".
pub fn installed_versions_in(versions_dir: &Path) -> Result<Vec<String>, CyoloError> {
    let entries = fs::read_dir(versions_dir).map_err(|e| CyoloError::ConfigIoError {
        context: format!("reading versions dir {}", versions_dir.display()),
        source: e,
    })?;
    let mut out = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| CyoloError::ConfigIoError {
            context: format!("reading versions dir {}", versions_dir.display()),
            source: e,
        })?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        out.push(name);
    }
    sort_versions(&mut out);
    Ok(out)
}

/// Sort version strings newest-first by numeric `(major, minor, patch)` key.
pub fn sort_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| version_key(b).cmp(&version_key(a)));
}

/// Numeric sort key for a dotted version. Each component contributes only its
/// leading digits, so a trailing tag like `-beta` never reorders the numeric
/// prefix; the raw string is the final tiebreaker for a stable, deterministic
/// ordering. Non-numeric input collapses to `(0, 0, 0, raw)`.
fn version_key(v: &str) -> (u64, u64, u64, String) {
    let mut parts = v.split('.').map(|p| {
        let digits: String = p.chars().take_while(|c| c.is_ascii_digit()).collect();
        digits.parse::<u64>().unwrap_or(0)
    });
    let major = parts.next().unwrap_or(0);
    let minor = parts.next().unwrap_or(0);
    let patch = parts.next().unwrap_or(0);
    (major, minor, patch, v.to_string())
}

/// Atomically repoint `bin_link` at `versions_dir/version`.
///
/// Validates the target exists first (we never create a dangling launcher),
/// then writes a temp symlink and renames it over the launcher so the swap is
/// atomic — a crash leaves either the old or the new link, never a
/// half-written one. A leftover temp link from a crashed prior run is cleared
/// before we create ours.
pub fn switch_in(bin_link: &Path, versions_dir: &Path, version: &str) -> Result<(), CyoloError> {
    let target = versions_dir.join(version);
    if !target.exists() {
        return Err(CyoloError::VersionNotInstalled {
            version: version.to_string(),
        });
    }
    let link_name = bin_link
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "claude".to_string());
    let tmp = bin_link.with_file_name(format!(".{link_name}.cyolo-tmp"));
    // Clear any stale temp link from a crashed prior run, else symlink() hits
    // EEXIST. Ignore "not found" — that's the happy path.
    let _ = fs::remove_file(&tmp);
    std::os::unix::fs::symlink(&target, &tmp).map_err(|e| CyoloError::ConfigIoError {
        context: format!("creating temp symlink {}", tmp.display()),
        source: e,
    })?;
    fs::rename(&tmp, bin_link).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        CyoloError::ConfigIoError {
            context: format!("repointing {}", bin_link.display()),
            source: e,
        }
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
