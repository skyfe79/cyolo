//! Creates `.claude-profile.json` in the current directory — the marker
//! file that walk-up resolution reads to decide which profile a given
//! directory binds to.
//!
//! Two shapes are supported by the marker schema (see `detect::ProfileFile`):
//!
//! * `{"name": "<registered profile>"}` — looked up via the global config.
//! * `{"config_dir": "<path>"}` — inline binding, no registration needed.
//!
//! Both variants are materialized through the same `write_marker` body so
//! the refuse-to-overwrite guard and the git-exclude side effect stay in
//! exactly one place.

use std::path::Path;

use owo_colors::OwoColorize;

use crate::error::CyoloError;

/// Write `.claude-profile.json` pointing at a registered profile `name`.
pub fn write_profile_marker(name: &str) -> Result<(), CyoloError> {
    write_marker(
        &serde_json::json!({"name": name}),
        &format!("profile: {}", name.green()),
    )
}

/// Write a `.claude-profile.json` that pins this directory to Claude Code's
/// own default config directory (`~/.claude`).
///
/// The tilde is kept literal in the file so the marker stays portable
/// across machines; `detect::resolve_profile` expands it at resolution time.
pub fn write_default_marker() -> Result<(), CyoloError> {
    write_marker(
        &serde_json::json!({"config_dir": "~/.claude"}),
        &format!("config_dir: {}", "~/.claude".green()),
    )
}

/// Shared body for [`write_profile_marker`] and [`write_default_marker`].
///
/// Refuses to overwrite an existing marker, writes the payload atomically,
/// and best-effort appends `.claude-profile.json` to `<gitdir>/info/exclude`
/// when the cwd sits inside a git repo. `label_suffix` is shown in the
/// success line's parentheses (e.g. `"profile: work"` or `"config_dir: ~/.claude"`).
fn write_marker(payload: &serde_json::Value, label_suffix: &str) -> Result<(), CyoloError> {
    let cwd = std::env::current_dir().map_err(|e| CyoloError::ConfigIoError {
        context: "could not determine current directory".into(),
        source: e,
    })?;
    let profile_path = cwd.join(".claude-profile.json");

    // `symlink_metadata` catches broken symlinks that `exists()` would miss.
    if std::fs::symlink_metadata(&profile_path).is_ok() {
        eprintln!(
            "{} .claude-profile.json already exists in {}",
            "error:".red().bold(),
            cwd.display()
        );
        return Err(CyoloError::NonZeroExit(1));
    }

    let contents = serde_json::to_string_pretty(payload)
        .expect("JSON serialization of simple object cannot fail");
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&profile_path)
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to create {}", profile_path.display()),
            source: e,
        })?;
    file.write_all(format!("{contents}\n").as_bytes())
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to write {}", profile_path.display()),
            source: e,
        })?;

    println!(
        "Created {} ({label_suffix})",
        ".claude-profile.json".green()
    );

    mark_gitignored(&cwd);

    Ok(())
}

/// Append `.claude-profile.json` to `<gitdir>/info/exclude` when the cwd
/// lives inside a git repository.  Silent on failure — marker creation has
/// already succeeded and we do not want an unrelated git-side hiccup to
/// change the command's exit code.
fn mark_gitignored(cwd: &Path) {
    if let Some(gitdir) = crate::git::find_gitdir(cwd)
        && let Ok(true) = crate::git::ensure_exclude_entry(&gitdir, ".claude-profile.json")
    {
        println!(
            "{} added {} to {}",
            "↳".dimmed(),
            ".claude-profile.json".dimmed(),
            format!("{}/info/exclude", gitdir.display()).dimmed()
        );
    }
}
