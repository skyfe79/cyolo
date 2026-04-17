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
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn find_gitdir_returns_directory_form() {
        let tmp = TempDir::new().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();

        let found = find_gitdir(tmp.path()).unwrap();
        assert_eq!(found, git);
    }

    #[test]
    fn find_gitdir_walks_up_from_subdirectory() {
        let tmp = TempDir::new().unwrap();
        let git = tmp.path().join(".git");
        fs::create_dir(&git).unwrap();
        let deep = tmp.path().join("a").join("b").join("c");
        fs::create_dir_all(&deep).unwrap();

        let found = find_gitdir(&deep).unwrap();
        assert_eq!(found, git);
    }

    #[test]
    fn find_gitdir_resolves_worktree_file_with_absolute_gitdir() {
        let tmp = TempDir::new().unwrap();
        let real_gitdir = tmp.path().join("shared-repo").join("worktrees").join("feature");
        fs::create_dir_all(&real_gitdir).unwrap();
        let worktree = tmp.path().join("worktree-checkout");
        fs::create_dir(&worktree).unwrap();
        fs::write(
            worktree.join(".git"),
            format!("gitdir: {}\n", real_gitdir.display()),
        )
        .unwrap();

        let found = find_gitdir(&worktree).unwrap();
        assert_eq!(found, real_gitdir);
    }

    #[test]
    fn find_gitdir_resolves_worktree_file_with_relative_gitdir() {
        let tmp = TempDir::new().unwrap();
        let worktree = tmp.path().join("worktree-checkout");
        fs::create_dir(&worktree).unwrap();
        // Relative path is interpreted against the .git file's parent (= worktree).
        fs::create_dir_all(worktree.join("nested").join("gitdir")).unwrap();
        fs::write(worktree.join(".git"), "gitdir: nested/gitdir\n").unwrap();

        let found = find_gitdir(&worktree).unwrap();
        assert_eq!(found, worktree.join("nested").join("gitdir"));
    }

    #[test]
    fn find_gitdir_returns_none_when_absent() {
        let tmp = TempDir::new().unwrap();
        let nested = tmp.path().join("a").join("b");
        fs::create_dir_all(&nested).unwrap();
        // Bounded walk-up: the temp dir has no .git anywhere in its ancestry.
        // However, this test is not *fully* hermetic — if some ancestor of the
        // temp dir happens to be a git repo, this would walk all the way up.
        // On CI that's usually fine; on a dev laptop where /tmp is outside any
        // repo the assertion holds reliably.
        let found = find_gitdir(&nested);
        // We can only reliably assert None when /tmp is not inside a repo.
        // If it is, tolerate that by insisting it's at least outside our temp tree.
        if let Some(path) = &found {
            assert!(
                !path.starts_with(tmp.path()),
                "unexpected gitdir inside tmp: {path:?}"
            );
        }
    }

    #[test]
    fn ensure_exclude_entry_creates_file_when_missing() {
        let tmp = TempDir::new().unwrap();
        let gitdir = tmp.path().to_path_buf();

        let added = ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap();
        assert!(added);

        let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
        assert_eq!(contents, ".claude-profile.json\n");
    }

    #[test]
    fn ensure_exclude_entry_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let gitdir = tmp.path().to_path_buf();

        assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());
        // Second call must not duplicate.
        assert!(!ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

        let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
        assert_eq!(contents.matches(".claude-profile.json").count(), 1);
    }

    #[test]
    fn ensure_exclude_entry_appends_after_missing_trailing_newline() {
        let tmp = TempDir::new().unwrap();
        let gitdir = tmp.path().to_path_buf();
        fs::create_dir_all(gitdir.join("info")).unwrap();
        // Pre-existing file with NO trailing newline — a common hand-edited shape.
        fs::write(gitdir.join("info").join("exclude"), "*.log").unwrap();

        assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

        let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
        assert_eq!(contents, "*.log\n.claude-profile.json\n");
    }

    #[test]
    fn ensure_exclude_entry_preserves_existing_content() {
        let tmp = TempDir::new().unwrap();
        let gitdir = tmp.path().to_path_buf();
        fs::create_dir_all(gitdir.join("info")).unwrap();
        fs::write(
            gitdir.join("info").join("exclude"),
            "# git ls-files --others --exclude-from=.git/info/exclude\n*.log\n",
        )
        .unwrap();

        assert!(ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());

        let contents = fs::read_to_string(gitdir.join("info").join("exclude")).unwrap();
        assert_eq!(
            contents,
            "# git ls-files --others --exclude-from=.git/info/exclude\n*.log\n.claude-profile.json\n"
        );
    }

    #[test]
    fn ensure_exclude_entry_detects_existing_with_whitespace() {
        let tmp = TempDir::new().unwrap();
        let gitdir = tmp.path().to_path_buf();
        fs::create_dir_all(gitdir.join("info")).unwrap();
        // Trailing spaces around an entry should still be recognised as a match.
        fs::write(
            gitdir.join("info").join("exclude"),
            "  .claude-profile.json  \n",
        )
        .unwrap();

        assert!(!ensure_exclude_entry(&gitdir, ".claude-profile.json").unwrap());
    }
}
