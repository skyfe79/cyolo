//! Tiny best-effort helpers for cooperating with an enclosing git repository.
//!
//! `cyolo` only needs two things from git:
//!
//! 1. locate the repo's `.git` directory — including the `.git`-as-file form
//!    that worktrees and submodules use;
//! 2. idempotently append a pathspec to `<gitdir>/info/exclude` so
//!    `.claude-profile.json` stays untracked without touching the committed
//!    `.gitignore`.
//!
//! Everything here is best-effort: any I/O failure should bubble up to the
//! caller as a regular `io::Result`, and the caller is expected to treat a
//! failure as "silently skip". We deliberately avoid shelling out to `git`
//! itself to keep `cyolo` dependency-free and offline-safe.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Walk up from `start` looking for a `.git` entry and return the actual
/// gitdir path (the directory that contains `HEAD`, `config`, `info/`, ...).
///
/// Behavior:
///   * `.git` as a directory → return that directory path.
///   * `.git` as a regular file → parse the first `gitdir: <path>` line.
///     Resolve the path as absolute, or relative to the parent of the `.git`
///     file. This covers `git worktree add` and submodule layouts.
///   * Nothing found → `None`.
pub fn find_gitdir(start: &Path) -> Option<PathBuf> {
    for ancestor in start.ancestors() {
        let git = ancestor.join(".git");
        let Ok(meta) = fs::symlink_metadata(&git) else {
            continue;
        };
        if meta.file_type().is_dir() {
            return Some(git);
        }
        if meta.file_type().is_file() {
            return parse_gitdir_file(&git);
        }
        // .git as a symlink: let std::fs::metadata (which follows) decide.
        if let Ok(resolved) = fs::metadata(&git) {
            if resolved.is_dir() {
                return Some(git);
            }
            if resolved.is_file() {
                return parse_gitdir_file(&git);
            }
        }
    }
    None
}

/// Parse the `gitdir: <path>` line that worktree/submodule `.git` files carry.
///
/// Returns the resolved gitdir path. Relative paths are interpreted against
/// the directory containing the `.git` file, which matches git's own rules.
fn parse_gitdir_file(git_file: &Path) -> Option<PathBuf> {
    let contents = fs::read_to_string(git_file).ok()?;
    let target = contents.lines().find_map(|l| l.strip_prefix("gitdir: "))?;
    let gd = PathBuf::from(target.trim());
    if gd.is_absolute() {
        Some(gd)
    } else {
        let parent = git_file.parent()?;
        Some(parent.join(gd))
    }
}

/// Idempotently append `entry` as its own line to `<gitdir>/info/exclude`.
///
/// Returns `Ok(true)` when a line was appended, `Ok(false)` when `entry` was
/// already present (exact match after trimming). Creates `info/` and
/// `exclude` on demand.
///
/// We compare trimmed lines so that stray whitespace does not double-count,
/// but we never normalize or rewrite existing content.
pub fn ensure_exclude_entry(gitdir: &Path, entry: &str) -> std::io::Result<bool> {
    let info_dir = gitdir.join("info");
    fs::create_dir_all(&info_dir)?;
    let exclude_path = info_dir.join("exclude");

    let existing = match fs::read_to_string(&exclude_path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => return Err(e),
    };

    if existing.lines().any(|l| l.trim() == entry.trim()) {
        return Ok(false);
    }

    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&exclude_path)?;

    // Guarantee the new entry starts on its own line even if the existing
    // file was not newline-terminated.
    if !existing.is_empty() && !existing.ends_with('\n') {
        file.write_all(b"\n")?;
    }
    file.write_all(entry.as_bytes())?;
    file.write_all(b"\n")?;
    Ok(true)
}


#[cfg(test)]
mod tests;
