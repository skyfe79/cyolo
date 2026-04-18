use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use clap::Parser;
use owo_colors::OwoColorize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;
use crate::util::expand_tilde;

// Clap shape for `cyolo diet [<flags>]`. Kept separate from `DietOptions`
// so the internal value type stays a plain `pub(crate)` struct without
// clap derives. Conversion happens in `DietCli::into_options` at the top
// of `dispatch`.
#[derive(Parser, Debug)]
#[command(
    name = "diet",
    about = "Report / reclaim orphaned project data + caches",
    no_binary_name = true,
    disable_version_flag = true
)]
pub struct DietCli {
    /// Actually remove orphaned entries + session folders (default: dry-run).
    #[arg(long)]
    pub apply: bool,
    /// Skip the "claude is running" guard.
    #[arg(long)]
    pub force: bool,
    /// Also include projects idle for ≥N days in the cleanup. Must be > 0.
    #[arg(long, value_name = "N")]
    pub stale_days: Option<u32>,
    /// Include cache directories (`statsig`, `shell-snapshots`, `file-history`).
    #[arg(long)]
    pub cache: bool,
    /// Operate on a specific registered profile.
    #[arg(long, conflicts_with = "all", value_name = "NAME")]
    pub profile: Option<String>,
    /// Iterate every registered profile instead of just the current one.
    #[arg(long)]
    pub all: bool,
}

impl DietCli {
    /// Convert the clap-parsed shape into the internal [`DietOptions`]
    /// value used throughout the rest of the module. Also enforces the
    /// `--stale-days > 0` constraint that clap alone can't express.
    fn into_options(self) -> Result<DietOptions, CyoloError> {
        if let Some(0) = self.stale_days {
            eprintln!(
                "{} --stale-days value must be greater than zero",
                "error:".red().bold()
            );
            return Err(CyoloError::NonZeroExit(1));
        }
        Ok(DietOptions {
            apply: self.apply,
            force: self.force,
            stale_days: self.stale_days,
            cache: self.cache,
            profile: self.profile,
            all: self.all,
        })
    }
}

/// Detect whether a Claude Code process is currently running.
///
/// Uses `pgrep -f "claude"` to check for any matching process.
/// Returns `false` if `pgrep` is not available or fails to execute.
pub(crate) fn is_claude_running() -> bool {
    Command::new("pgrep")
        .args(["-f", "claude"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Parsed CLI arguments for the `diet` subcommand.
#[derive(Debug)]
pub(crate) struct DietOptions {
    /// Whether `--apply` was provided (execute cleanup vs dry-run).
    pub apply: bool,
    /// Whether `--force` was provided (skip safety checks like Claude-running guard).
    pub force: bool,
    pub stale_days: Option<u32>,
    pub cache: bool,
    pub profile: Option<String>,
    pub all: bool,
}

/// A project entry in `~/.claude.json` whose filesystem path no longer exists.
#[derive(Debug, Clone)]
pub(crate) struct OrphanedProject {
    /// Absolute filesystem path from the `~/.claude.json` projects key.
    pub path: String,
    /// Approximate size of this entry's serialized JSON in bytes.
    pub entry_size: u64,
}

/// A session folder in `~/.claude/projects/` that belongs to an orphaned project.
#[derive(Debug)]
pub(crate) struct OrphanedSession {
    /// Full path to the session folder.
    pub folder_path: PathBuf,
    /// Sum of all file sizes within the folder.
    pub total_size: u64,
}

/// A project whose session files have not been modified within the staleness threshold.
#[derive(Debug)]
pub(crate) struct StaleProject {
    /// Absolute filesystem path of the project.
    pub path: String,
    /// Seconds since the newest file in the session directory was modified.
    pub last_activity_secs: u64,
    /// Approximate serialized size of this project's `history` array in the JSON config.
    pub history_size: u64,
    /// Total size of the project's session directory in bytes.
    pub session_size: u64,
}

/// A cache directory inside `~/.claude/` whose contents can be safely removed.
#[derive(Debug)]
pub(crate) struct CacheDir {
    /// Human-readable name of the cache directory (e.g. "statsig").
    pub name: String,
    /// Full path to the cache directory.
    pub path: PathBuf,
    /// Total size of all files within the directory in bytes.
    pub size: u64,
}

/// Aggregated analysis results for the diet command.
#[derive(Debug)]
pub(crate) struct DietReport {
    /// Projects in `~/.claude.json` whose paths no longer exist.
    pub orphaned_projects: Vec<OrphanedProject>,
    /// Session folders in `~/.claude/projects/` that map to orphaned projects.
    pub orphaned_sessions: Vec<OrphanedSession>,
    /// Projects whose session files have not been modified within the staleness threshold.
    pub stale_projects: Vec<StaleProject>,
    /// Cache directories inside `~/.claude/` whose contents can be removed.
    pub cache_dirs: Vec<CacheDir>,
    /// Number of projects whose paths still exist.
    pub active_project_count: usize,
    /// Total size of `~/.claude.json` in bytes.
    pub config_file_size: u64,
    /// Total size of all contents in `~/.claude/projects/`.
    pub session_dir_total_size: u64,
    /// Path to `~/.claude`.
    pub claude_home: PathBuf,
}

/// Results from analyzing `~/.claude.json` for orphaned projects.
///
/// Returned by [`analyze_claude_json`]. Keeps the parsed `serde_json::Value`
/// so that callers (e.g. apply) can reuse it without re-parsing.
#[derive(Debug)]
pub(crate) struct AnalysisResult {
    /// Projects whose filesystem paths no longer exist.
    pub orphaned_projects: Vec<OrphanedProject>,
    /// Number of projects whose paths still exist.
    pub active_count: usize,
    /// Total size of `~/.claude.json` in bytes.
    pub config_file_size: u64,
    /// The full parsed JSON, preserved for apply to mutate and write back.
    pub parsed_json: serde_json::Value,
}

/// Analyze `~/.claude.json` and identify orphaned projects.
///
/// Gracefully handles:
/// - Missing file → empty results with `Value::Null`
/// - Missing `projects` key → empty results
/// - Empty `projects` object → zero orphans, zero active
pub(crate) fn analyze_claude_json(claude_json_path: &Path) -> Result<AnalysisResult, CyoloError> {
    if !claude_json_path.exists() {
        return Ok(AnalysisResult {
            orphaned_projects: Vec::new(),
            active_count: 0,
            config_file_size: 0,
            parsed_json: serde_json::Value::Null,
        });
    }

    let config_file_size = std::fs::metadata(claude_json_path)
        .map(|m| m.len())
        .unwrap_or(0);

    let content =
        std::fs::read_to_string(claude_json_path).map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to read {}", claude_json_path.display()),
            source: e,
        })?;

    let parsed: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| CyoloError::ConfigParseError {
            path: claude_json_path.to_path_buf(),
            source: e,
        })?;

    let projects = match parsed.get("projects").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => {
            return Ok(AnalysisResult {
                orphaned_projects: Vec::new(),
                active_count: 0,
                config_file_size,
                parsed_json: parsed,
            });
        }
    };

    let mut orphaned = Vec::new();
    let mut active_count = 0;

    for (path_key, value) in projects {
        if Path::new(path_key).exists() {
            active_count += 1;
        } else {
            let entry_size = serde_json::to_string(value).unwrap_or_default().len() as u64;
            orphaned.push(OrphanedProject {
                path: path_key.clone(),
                entry_size,
            });
        }
    }

    Ok(AnalysisResult {
        orphaned_projects: orphaned,
        active_count,
        config_file_size,
        parsed_json: parsed,
    })
}

/// Format a byte count as a human-readable string (B, KB, MB, GB).
///
/// Uses 1024-based divisions with one decimal place for KB and above.
/// Promotes to the next unit when rounding would display "1024.0".
pub(crate) fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    // Threshold above which {:.1} rounds to 1024.0, requiring unit promotion.
    const PROMOTE: f64 = 1023.95;

    let b = bytes as f64;
    if bytes == 0 {
        "0 B".to_string()
    } else if b < KB {
        format!("{bytes} B")
    } else if b < MB && b / KB < PROMOTE {
        format!("{:.1} KB", b / KB)
    } else if b < GB && b / MB < PROMOTE {
        format!("{:.1} MB", b / MB)
    } else {
        format!("{:.1} GB", b / GB)
    }
}

/// Convert a project filesystem path to its session directory name.
///
/// Claude stores per-project session data in `~/.claude/projects/` using the
/// project's absolute path with `/` replaced by `-`.
/// e.g. `/Users/codingmax/Private/labs/test-bot` → `-Users-codingmax-Private-labs-test-bot`
pub(crate) fn project_path_to_session_dir(project_path: &str) -> String {
    project_path.replace('/', "-")
}

/// Calculate the total size (in bytes) of all files within a directory, recursively.
///
/// Returns 0 if the path does not exist or is not a directory.
/// Silently skips entries that cannot be read (e.g. permission errors).
/// Symlinks are not followed — they are skipped to avoid loops and counting
/// data outside the session tree.
pub(crate) fn dir_size(path: &Path) -> u64 {
    // Use symlink_metadata so symlinks themselves don't fool the is_dir check.
    let meta = match fs::symlink_metadata(path) {
        Ok(m) => m,
        Err(_) => return 0,
    };
    if !meta.is_dir() {
        return 0;
    }

    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(_) => return 0,
    };

    let mut total: u64 = 0;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        // Use symlink_metadata to avoid following symlinks.
        if let Ok(entry_meta) = fs::symlink_metadata(&entry_path) {
            if entry_meta.is_dir() {
                total += dir_size(&entry_path);
            } else if entry_meta.is_file() {
                total += entry_meta.len();
            }
            // Symlinks (entry_meta.is_symlink()) are intentionally skipped.
        }
    }
    total
}

/// Scan session folders in `projects_dir` and identify those belonging to orphaned projects.
///
/// Returns a tuple of:
/// - `Vec<OrphanedSession>`: session folders that map to orphaned project paths
/// - `u64`: total size of *all* session folders in `projects_dir`
pub(crate) fn scan_session_folders(
    projects_dir: &Path,
    orphaned_paths: &[String],
) -> (Vec<OrphanedSession>, u64) {
    if !projects_dir.exists() {
        return (Vec::new(), 0);
    }

    // Calculate total size of ALL entries in the projects directory.
    let mut total_session_dir_size: u64 = 0;
    if let Ok(entries) = fs::read_dir(projects_dir) {
        for entry in entries.flatten() {
            total_session_dir_size += dir_size(&entry.path());
        }
    }

    // Find session folders that correspond to orphaned project paths.
    // Use symlink_metadata to avoid following a symlinked session folder root.
    let mut orphaned_sessions = Vec::new();
    for path in orphaned_paths {
        let session_name = project_path_to_session_dir(path);
        let session_path = projects_dir.join(&session_name);
        if let Ok(meta) = fs::symlink_metadata(&session_path)
            && meta.is_dir()
        {
            let total_size = dir_size(&session_path);
            orphaned_sessions.push(OrphanedSession {
                folder_path: session_path,
                total_size,
            });
        }
        // If it's a symlink (not a real dir), skip — don't follow it.
    }

    (orphaned_sessions, total_session_dir_size)
}

/// Find the newest modification time among all files in a directory tree.
///
/// Mirrors the [`dir_size`] recursion pattern (symlink-safe via `fs::symlink_metadata`).
/// Returns `None` if no file has a valid `modified()` timestamp.
fn newest_mtime(dir: &Path) -> Option<SystemTime> {
    let meta = match fs::symlink_metadata(dir) {
        Ok(m) => m,
        Err(_) => return None,
    };
    if !meta.is_dir() {
        return None;
    }

    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return None,
    };

    let mut newest: Option<SystemTime> = None;
    for entry in entries.flatten() {
        let entry_path = entry.path();
        // Use symlink_metadata to avoid following symlinks.
        if let Ok(entry_meta) = fs::symlink_metadata(&entry_path) {
            if entry_meta.is_dir() {
                if let Some(child_mtime) = newest_mtime(&entry_path) {
                    newest = Some(match newest {
                        Some(current) if current >= child_mtime => current,
                        _ => child_mtime,
                    });
                }
            } else if entry_meta.is_file() {
                match entry_meta.modified() {
                    Ok(mtime) => {
                        newest = Some(match newest {
                            Some(current) if current >= mtime => current,
                            _ => mtime,
                        });
                    }
                    Err(e) => {
                        eprintln!(
                            "{} could not read mtime for {}: {}",
                            "warning:".yellow().bold(),
                            entry_path.display(),
                            e
                        );
                    }
                }
            }
            // Symlinks (entry_meta.is_symlink()) are intentionally skipped.
        }
    }
    newest
}

/// Detect projects whose session files have not been modified within the staleness threshold.
///
/// Only considers projects whose filesystem path still exists on disk (orphaned projects
/// are handled separately). Projects with no session directory or an empty session directory
/// are given the benefit of the doubt and skipped.
pub(crate) fn detect_stale_projects(
    parsed_json: &serde_json::Value,
    projects_dir: &Path,
    stale_days: u32,
) -> Vec<StaleProject> {
    let projects = match parsed_json.get("projects").and_then(|v| v.as_object()) {
        Some(obj) => obj,
        None => return Vec::new(),
    };

    let threshold_secs = stale_days as u64 * 86400;
    let now = SystemTime::now();
    let mut stale = Vec::new();

    for (path_key, _project_value) in projects {
        // Only consider projects whose filesystem path still exists.
        if !Path::new(path_key).exists() {
            continue;
        }

        let session_dir_name = project_path_to_session_dir(path_key);
        let session_path = projects_dir.join(&session_dir_name);

        // Check if session dir exists AND is a real directory (skip symlinks).
        match fs::symlink_metadata(&session_path) {
            Ok(meta) if meta.is_dir() => {}
            _ => continue, // No session dir or not a real dir → skip.
        }

        // Empty session dir or no valid mtime → skip (benefit of the doubt).
        let mtime = match newest_mtime(&session_path) {
            Some(t) => t,
            None => continue,
        };

        // If mtime is in the future, duration_since returns Err → not stale.
        let age = match now.duration_since(mtime) {
            Ok(d) => d,
            Err(_) => continue,
        };

        if age.as_secs() >= threshold_secs {
            // Compute history_size from the JSON (only if history is an array).
            let history_size = parsed_json
                .get("projects")
                .and_then(|p| p.get(path_key))
                .and_then(|proj| proj.get("history"))
                .filter(|h| h.is_array())
                .and_then(|h| serde_json::to_string(h).ok())
                .map(|s| s.len() as u64)
                .unwrap_or(0);

            let session_size = dir_size(&session_path);

            stale.push(StaleProject {
                path: path_key.clone(),
                last_activity_secs: age.as_secs(),
                history_size,
                session_size,
            });
        }
    }

    stale
}

/// Measure cache directories inside `claude_home` that can be safely cleaned.
///
/// Checks three known cache directories: `statsig/`, `shell-snapshots/`, and
/// `file-history/`. Returns entries only for directories that exist on disk.
pub(crate) fn measure_cache_dirs(claude_home: &Path) -> Vec<CacheDir> {
    const CACHE_NAMES: &[&str] = &["statsig", "shell-snapshots", "file-history"];

    let mut result = Vec::new();
    for &name in CACHE_NAMES {
        let path = claude_home.join(name);
        match fs::symlink_metadata(&path) {
            Ok(meta) if meta.is_dir() => {
                let size = dir_size(&path);
                result.push(CacheDir {
                    name: name.to_string(),
                    path,
                    size,
                });
            }
            _ => {}
        }
    }
    result
}

/// Remove all contents within each cache directory, preserving the directories themselves.
///
/// For each `CacheDir`: iterates `fs::read_dir` entries, uses `fs::remove_dir_all`
/// for subdirectories and `fs::remove_file` for files. Individual removal failures
/// warn to stderr but don't abort.
/// Returns `(removed_count, bytes_freed)` where `removed_count` is the number of
/// cache directories that were processed and `bytes_freed` is the sum of their sizes.
pub(crate) fn remove_cache_contents(
    cache_dirs: &[CacheDir],
) -> Result<(usize, u64), CyoloError> {
    let mut removed_count: usize = 0;
    let mut bytes_freed: u64 = 0;

    for cache in cache_dirs {
        // Re-check that the cache path is still a real directory (not a symlink)
        // to guard against a TOCTOU race where the dir was replaced between
        // measure_cache_dirs() and this call.
        match fs::symlink_metadata(&cache.path) {
            Ok(meta) if meta.is_dir() => {}
            _ => {
                eprintln!(
                    "{} cache directory {} is no longer a real directory, skipping",
                    "warning:".yellow().bold(),
                    cache.path.display()
                );
                continue;
            }
        }

        let entries = match fs::read_dir(&cache.path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!(
                    "{} could not read cache directory {}: {}",
                    "warning:".yellow().bold(),
                    cache.path.display(),
                    e
                );
                continue;
            }
        };

        for entry_result in entries {
            let entry = match entry_result {
                Ok(e) => e,
                Err(e) => {
                    eprintln!(
                        "{} could not read entry in {}: {}",
                        "warning:".yellow().bold(),
                        cache.path.display(),
                        e
                    );
                    continue;
                }
            };
            let entry_path = entry.path();
            let entry_meta = match fs::symlink_metadata(&entry_path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "{} could not stat {}: {}",
                        "warning:".yellow().bold(),
                        entry_path.display(),
                        e
                    );
                    continue;
                }
            };

            let result = if entry_meta.is_dir() {
                fs::remove_dir_all(&entry_path)
            } else {
                fs::remove_file(&entry_path)
            };

            if let Err(e) = result {
                eprintln!(
                    "{} failed to remove {}: {}",
                    "warning:".yellow().bold(),
                    entry_path.display(),
                    e
                );
            }
        }

        removed_count += 1;
        bytes_freed += cache.size;
    }

    Ok((removed_count, bytes_freed))
}

/// Replace the user's home directory prefix with `~` for shorter display paths.
///
/// Uses `Path::strip_prefix` for boundary-safe matching (avoids false positives
/// like `/Users/codingmax-old/` being rewritten to `~-old/`).
fn tilde_path(path: &str) -> String {
    if let Some(home) = dirs::home_dir()
        && let Ok(rest) = Path::new(path).strip_prefix(&home)
    {
        let rest_str = rest.to_string_lossy();
        if rest_str.is_empty() {
            return "~".to_string();
        }
        return format!("~/{rest_str}");
    }
    path.to_string()
}

/// Build a tree-style report string from the diet analysis results.
///
/// Returns a `String` (rather than printing directly) for testability.
/// `applied` controls the footer text ("Cleanup complete." vs "Run with --apply").
pub(crate) fn build_report_string(report: &DietReport, applied: bool) -> String {
    let claude_home_display = tilde_path(&report.claude_home.to_string_lossy());

    // Special case: nothing to clean up at all.
    if report.orphaned_projects.is_empty()
        && report.orphaned_sessions.is_empty()
        && report.stale_projects.is_empty()
        && report.cache_dirs.is_empty()
    {
        return format!(
            "{}\n\nNo orphaned projects found. Nothing to clean up.\n",
            format!("cyolo diet — analyzing {claude_home_display}").bold()
        );
    }

    let mut buf = String::new();

    // Header
    writeln!(
        buf,
        "{}",
        format!("cyolo diet — analyzing {claude_home_display}").bold()
    )
    .unwrap();
    writeln!(buf).unwrap();

    // ── ~/.claude.json section ──────────────────────────────────
    let claude_json_display = format!("{claude_home_display}.json");
    let orphan_total_size: u64 = report.orphaned_projects.iter().map(|o| o.entry_size).sum();

    let claude_json_label = format!("{:<45}", format!("{claude_json_display}:"));
    writeln!(
        buf,
        "{} {}",
        claude_json_label.bold(),
        format_size(report.config_file_size).cyan()
    )
    .unwrap();

    let orphan_count = report.orphaned_projects.len();

    // Orphaned projects line (always shown)
    writeln!(
        buf,
        "  {} {:<41} {}  {}",
        "├─".dimmed(),
        format!("orphaned projects ({orphan_count}):"),
        format_size(orphan_total_size).cyan(),
        "(removable)".green()
    )
    .unwrap();

    if orphan_count > 0 {
        // Individual orphaned paths (max 5)
        let display_count = orphan_count.min(5);
        for (i, orphan) in report.orphaned_projects.iter().take(5).enumerate() {
            let is_last = i == display_count - 1 && orphan_count <= 5;
            let item_char = if is_last { "└─" } else { "├─" };
            writeln!(
                buf,
                "  {}   {} {:<37} {}",
                "│".dimmed(),
                item_char.dimmed(),
                tilde_path(&orphan.path),
                format_size(orphan.entry_size).cyan()
            )
            .unwrap();
        }
        if orphan_count > 5 {
            writeln!(
                buf,
                "  {}   {} ... {} more",
                "│".dimmed(),
                "└─".dimmed(),
                orphan_count - 5
            )
            .unwrap();
        }
    }

    // Active projects line (always shown)
    let active_size = report.config_file_size.saturating_sub(orphan_total_size);
    writeln!(
        buf,
        "  {} {:<41} {}  {}",
        "└─".dimmed(),
        format!("active projects ({}):", report.active_project_count),
        format_size(active_size).cyan(),
        "(keep)".dimmed()
    )
    .unwrap();

    writeln!(buf).unwrap();

    // ── Stale projects section (after orphans) ─────────────────
    if !report.stale_projects.is_empty() {
        let stale_count = report.stale_projects.len();
        let stale_history_total: u64 = report.stale_projects.iter().map(|s| s.history_size).sum();

        let stale_header = format!("{:<45}", format!("stale projects ({stale_count}):"));
        writeln!(
            buf,
            "{} {}  {}",
            stale_header.bold(),
            format_size(stale_history_total).cyan(),
            "(history clearable)".yellow()
        )
        .unwrap();

        for (i, stale) in report.stale_projects.iter().enumerate() {
            let is_last = i == stale_count - 1;
            let item_char = if is_last { "└─" } else { "├─" };
            let days = stale.last_activity_secs / 86400;
            writeln!(
                buf,
                "  {} {:<30} last activity: {} days ago     {}",
                item_char.dimmed(),
                tilde_path(&stale.path),
                days,
                format_size(stale.history_size).cyan()
            )
            .unwrap();
        }

        writeln!(buf).unwrap();
    }

    // ── ~/.claude/projects/ section ─────────────────────────────
    let projects_header = format!("{:<45}", format!("{claude_home_display}/projects/:"));
    writeln!(
        buf,
        "{} {}",
        projects_header.bold(),
        format_size(report.session_dir_total_size).cyan()
    )
    .unwrap();

    let session_total_size: u64 = report.orphaned_sessions.iter().map(|s| s.total_size).sum();
    let session_count = report.orphaned_sessions.len();
    // Always show orphaned session folders line
    writeln!(
        buf,
        "  {} {:<41} {}  {}",
        "└─".dimmed(),
        format!("orphaned session folders ({session_count}):"),
        format_size(session_total_size).cyan(),
        "(removable)".green()
    )
    .unwrap();

    writeln!(buf).unwrap();

    // ── Cache directories section (after sessions) ─────────────
    if !report.cache_dirs.is_empty() {
        let cache_total: u64 = report.cache_dirs.iter().map(|c| c.size).sum();
        let cache_count = report.cache_dirs.len();

        let cache_header = format!("{:<45}", format!("{claude_home_display}/cache:"));
        writeln!(
            buf,
            "{} {}",
            cache_header.bold(),
            format_size(cache_total).cyan()
        )
        .unwrap();

        writeln!(
            buf,
            "  {} {:<41} {}  {}",
            "└─".dimmed(),
            format!("clearable cache dirs ({cache_count}):"),
            format_size(cache_total).cyan(),
            "(removable)".green()
        )
        .unwrap();

        for (i, cache) in report.cache_dirs.iter().enumerate() {
            let is_last = i == cache_count - 1;
            let item_char = if is_last { "└─" } else { "├─" };
            writeln!(
                buf,
                "      {} {:<37} {}",
                item_char.dimmed(),
                format!("{}/", cache.name),
                format_size(cache.size).cyan()
            )
            .unwrap();
        }

        writeln!(buf).unwrap();
    }

    // Total reclaimable (all categories)
    let stale_history_total: u64 = report.stale_projects.iter().map(|s| s.history_size).sum();
    let stale_session_total: u64 = report.stale_projects.iter().map(|s| s.session_size).sum();
    let cache_total: u64 = report.cache_dirs.iter().map(|c| c.size).sum();
    let total_reclaimable =
        orphan_total_size + session_total_size + stale_history_total + stale_session_total + cache_total;
    writeln!(
        buf,
        "{} {}",
        "Total reclaimable:".bold(),
        format_size(total_reclaimable).cyan()
    )
    .unwrap();
    writeln!(buf).unwrap();

    // Footer
    if applied {
        writeln!(buf, "Cleanup complete.").unwrap();
    } else {
        writeln!(buf, "Run with --apply to proceed.").unwrap();
    }

    buf
}

/// Print the diet analysis report to stdout.
pub(crate) fn print_report(report: &DietReport, applied: bool) {
    print!("{}", build_report_string(report, applied));
}

/// Format a Unix timestamp (seconds since epoch) as `YYYYMMDDHHMMSS`.
///
/// Uses Howard Hinnant's `civil_from_days` algorithm for correct leap-year handling
/// without any external dependency (no chrono).
pub(crate) fn format_timestamp(secs: u64) -> String {
    let days = (secs / 86400) as i64 + 719_468; // shift epoch to 0000-03-01
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let doe = (days - era * 146_097) as u64; // day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365; // year of era
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year [0, 365]
    let mp = (5 * doy + 2) / 153; // month index [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };

    let rem = secs % 86400;
    let h = rem / 3600;
    let min = (rem % 3600) / 60;
    let s = rem % 60;

    format!("{:04}{:02}{:02}{:02}{:02}{:02}", y, m, d, h, min, s)
}

/// Create a timestamped backup of `~/.claude.json`.
///
/// Returns the path to the newly created backup file.
pub(crate) fn backup_claude_json(claude_json_path: &Path) -> Result<PathBuf, CyoloError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let ts = format_timestamp(now);

    let file_name = format!(
        "{}.backup-{ts}",
        claude_json_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );
    let backup_path = claude_json_path.with_file_name(file_name);

    fs::copy(claude_json_path, &backup_path).map_err(|e| CyoloError::ConfigIoError {
        context: format!(
            "failed to back up {} to {}",
            claude_json_path.display(),
            backup_path.display()
        ),
        source: e,
    })?;

    Ok(backup_path)
}

/// Rotate backup files for a given path, keeping only the most recent `keep` backups.
///
/// Lists files matching `<filename>.backup-*` in the parent directory, sorts
/// alphabetically (YYYYMMDDHHMMSS naturally sorts chronologically), keeps the
/// latest `keep` entries, and deletes the rest. Individual delete failures warn
/// to stderr but don't abort.
pub(crate) fn rotate_backups(original_path: &Path, keep: usize) -> Result<(), CyoloError> {
    let parent = match original_path.parent() {
        Some(p) => p,
        None => return Ok(()),
    };

    let prefix = format!(
        "{}.backup-",
        original_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
    );

    let entries = fs::read_dir(parent).map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to read directory {}", parent.display()),
        source: e,
    })?;

    let mut backups: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with(&prefix)
        })
        .map(|entry| entry.path())
        .collect();

    backups.sort();

    let count = backups.len();
    if count <= keep {
        return Ok(());
    }

    for backup in &backups[..count - keep] {
        if let Err(err) = fs::remove_file(backup) {
            eprintln!(
                "{} failed to remove old backup {}: {}",
                "warning:".yellow().bold(),
                backup.display(),
                err
            );
        }
    }

    Ok(())
}

/// Write JSON content to a file atomically using temp-file + sync + rename.
///
/// Creates a temporary file (`.json.tmp` extension) in the same directory,
/// writes `content`, calls `sync_all()` to flush to disk, then atomically
/// renames over the target `path`. This prevents corruption if the process
/// crashes mid-write. Mirrors the pattern used in `config.rs::save()`.
fn atomic_write_json(path: &Path, content: &str) -> Result<(), CyoloError> {
    let tmp_path = path.with_extension("json.tmp");

    let mut file = fs::File::create(&tmp_path).map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to create temp file {}", tmp_path.display()),
        source: e,
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to write temp file {}", tmp_path.display()),
            source: e,
        })?;

    file.sync_all().map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to sync temp file {}", tmp_path.display()),
        source: e,
    })?;

    fs::rename(&tmp_path, path).map_err(|e| CyoloError::ConfigIoError {
        context: format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            path.display()
        ),
        source: e,
    })?;

    // Sync the parent directory so the rename is durable across power loss.
    if let Some(parent) = path.parent() {
        let dir = fs::File::open(parent).map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to open parent dir {}", parent.display()),
            source: e,
        })?;
        dir.sync_all().map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to sync parent dir {}", parent.display()),
            source: e,
        })?;
    }

    Ok(())
}

/// Clear history arrays for stale projects in memory.
///
/// For each path in `stale_paths`, sets `parsed_json["projects"][path]["history"]`
/// to an empty array if the existing value is an array. Warns to stderr and skips
/// if the value is not an array. Does NOT write to disk — the caller is responsible
/// for persisting the mutation.
pub(crate) fn clear_stale_history(
    parsed_json: &mut serde_json::Value,
    stale_paths: &[String],
) {
    let projects = match parsed_json
        .get_mut("projects")
        .and_then(|v| v.as_object_mut())
    {
        Some(obj) => obj,
        None => return,
    };

    for path in stale_paths {
        let entry = match projects.get_mut(path.as_str()) {
            Some(v) => v,
            None => continue,
        };

        let history = match entry.get_mut("history") {
            Some(v) => v,
            None => continue,
        };

        if history.is_array() {
            *history = serde_json::Value::Array(vec![]);
        } else {
            eprintln!(
                "{} history for {} is not an array, skipping",
                "warning:".yellow().bold(),
                path
            );
        }
    }
}

/// Remove orphaned project entries from the parsed JSON (in-memory only).
///
/// Uses `serde_json::Map::retain` with a `HashSet` for O(1) lookup.
/// Does NOT write to disk — the caller is responsible for persisting the mutation.
pub(crate) fn remove_orphaned_entries(
    parsed_json: &mut serde_json::Value,
    orphaned_paths: &[String],
) {
    let orphaned_set: HashSet<&str> = orphaned_paths.iter().map(|s| s.as_str()).collect();

    if let Some(projects) = parsed_json
        .get_mut("projects")
        .and_then(|v| v.as_object_mut())
    {
        projects.retain(|key, _| !orphaned_set.contains(key.as_str()));
    }
}

/// Remove orphaned session folders with symlink safety.
///
/// For each folder: iterate top-level entries, unlink any symlinks with
/// `fs::remove_file` (never follow), then `fs::remove_dir_all` for the folder.
/// Returns `(removed_count, bytes_freed)`.
pub(crate) fn remove_session_folders(
    sessions: &[OrphanedSession],
) -> Result<(usize, u64), CyoloError> {
    let mut removed_count: usize = 0;
    let mut bytes_freed: u64 = 0;

    for session in sessions {
        // Guard: if the session folder root itself is a symlink, just unlink it.
        // Never read_dir or remove_dir_all through a symlinked root — that would
        // mutate files outside ~/.claude/projects/.
        let root_meta = fs::symlink_metadata(&session.folder_path).map_err(|e| {
            CyoloError::ConfigIoError {
                context: format!(
                    "failed to stat session folder {}",
                    session.folder_path.display()
                ),
                source: e,
            }
        })?;

        if root_meta.file_type().is_symlink() {
            fs::remove_file(&session.folder_path).map_err(|e| CyoloError::ConfigIoError {
                context: format!(
                    "failed to unlink symlinked session folder {}",
                    session.folder_path.display()
                ),
                source: e,
            })?;
        } else {
            // First pass: unlink any symlinks in the top-level directory entries.
            if let Ok(entries) = fs::read_dir(&session.folder_path) {
                for entry in entries.flatten() {
                    if let Ok(meta) = fs::symlink_metadata(entry.path())
                        && meta.file_type().is_symlink()
                    {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }

            // Second pass: remove the entire folder (only regular files/dirs remain).
            fs::remove_dir_all(&session.folder_path).map_err(|e| CyoloError::ConfigIoError {
                context: format!(
                    "failed to remove session folder {}",
                    session.folder_path.display()
                ),
                source: e,
            })?;
        }

        removed_count += 1;
        bytes_freed += session.total_size;
    }

    Ok((removed_count, bytes_freed))
}

/// Parse CLI arguments for the `diet` subcommand.
///
/// No args → dry-run.
/// `--apply` → execute cleanup.
/// `--force` → skip safety checks.
/// `--stale-days <N>` → only consider entries older than N days (N > 0).
/// `--cache` → include cache cleanup.
/// `--profile <name>` → target a specific profile (mutually exclusive with `--all`).
/// `--all` → target all profiles (mutually exclusive with `--profile`).
/// Order-independent. Unknown args → error with usage message.
/// Thin wrapper around [`DietCli`] that kept the same signature for tests
/// across the clap migration. New code should prefer `DietCli::try_parse_from`
/// + `into_options` directly.
///
/// `#[cfg(test)]`-gated because `dispatch` now parses through `DietCli`
/// directly — this wrapper only exists to keep the pre-migration test
/// suite compiling unchanged. Release builds don't need it.
#[cfg(test)]
fn parse_diet_args(args: &[String]) -> Result<DietOptions, CyoloError> {
    let cli = DietCli::try_parse_from(args).map_err(|e| {
        // Print the clap error for user visibility (tests capture stderr),
        // then collapse to a NonZeroExit so callers don't need to reason
        // about clap's exit-code semantics.
        e.print().ok();
        CyoloError::NonZeroExit(e.exit_code())
    })?;
    cli.into_options()
}

/// Turn a `clap::Error` into either `Ok(())` (help/version display) or
/// `Err(NonZeroExit(exit_code))` (genuine parse failure). Mirrors the
/// `profile::handle_clap_error` helper.
fn handle_clap_error(e: clap::Error) -> Result<(), CyoloError> {
    use clap::error::ErrorKind;
    match e.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            e.print().ok();
            Ok(())
        }
        _ => {
            e.print().ok();
            Err(CyoloError::NonZeroExit(e.exit_code()))
        }
    }
}

/// Resolve the user's home directory and Claude home directory (`~/.claude`).
///
/// Returns `(home_dir, claude_home)`.
fn resolve_claude_home() -> Result<(PathBuf, PathBuf), CyoloError> {
    let home = dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
        context: "could not determine home directory".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
    })?;
    let claude_home = home.join(".claude");
    Ok((home, claude_home))
}

/// Determine which profile(s) to target based on CLI flags.
///
/// Returns a list of `(display_name, claude_home_path)` tuples:
/// - `--profile <name>`: single named profile from `~/.cyolo/config.json`.
/// - `--all`: every registered profile.
/// - Neither: the current profile via `resolve_claude_home()`.
pub(crate) fn resolve_target_profiles(
    options: &DietOptions,
) -> Result<Vec<(String, PathBuf)>, CyoloError> {
    if options.profile.is_some() || options.all {
        let cfg = CyoloConfig::load()?;
        return resolve_profiles_from_config(options, cfg);
    }

    // Default: current profile detection via resolve_claude_home().
    let (_home, claude_home) = resolve_claude_home()?;
    let display = tilde_path(&claude_home.to_string_lossy());
    Ok(vec![(display, claude_home)])
}

/// Core profile resolution logic, separated for testability.
fn resolve_profiles_from_config(
    options: &DietOptions,
    cfg: CyoloConfig,
) -> Result<Vec<(String, PathBuf)>, CyoloError> {
    if let Some(ref name) = options.profile {
        let profile = cfg.profiles.get(name).ok_or_else(|| {
            CyoloError::ProfileNotFound { name: name.clone() }
        })?;
        let dir = expand_tilde(&profile.config_dir.to_string_lossy());
        return Ok(vec![(name.clone(), dir)]);
    }

    // options.all
    if cfg.profiles.is_empty() {
        eprintln!(
            "{} no profiles registered. Run: cyolo profile add <name>",
            "error:".red().bold()
        );
        return Err(CyoloError::NonZeroExit(1));
    }
    let entries: Vec<(String, PathBuf)> = cfg
        .profiles
        .into_iter()
        .map(|(name, profile)| {
            let dir = expand_tilde(&profile.config_dir.to_string_lossy());
            (name, dir)
        })
        .collect();
    Ok(entries)
}

/// Entry point for the `diet` subcommand.
///
/// Orchestrates the full pipeline:
/// 1. Parse args → safety check → resolve profiles
/// 2. Parse `~/.claude.json` once (shared across all profiles)
/// 3. FOR each profile: orphan analysis, stale detection, cache measurement, session scan, report
/// 4. IF apply (after loop): backup, rotate, mutate JSON (orphans + stale), single atomic write,
///    then per-profile session/cache cleanup
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    // Parse via clap; `--help` and friends are handled by `handle_clap_error`.
    let cli = match DietCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => return handle_clap_error(e),
    };
    let options = cli.into_options()?;

    if !options.force && is_claude_running() {
        eprintln!(
            "{} Claude is currently running. Stop Claude first, or use --force to proceed.",
            "error:".red().bold()
        );
        return Err(CyoloError::NonZeroExit(1));
    }

    // Resolve target profiles.
    let profiles = resolve_target_profiles(&options)?;
    let multi_profile = profiles.len() > 1;

    // ~/.claude.json is always at $HOME/.claude.json regardless of profile.
    let home = dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
        context: "could not determine home directory".into(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
    })?;
    let claude_json_path = home.join(".claude.json");
    let mut analysis = analyze_claude_json(&claude_json_path)?;

    // Accumulators for the post-loop apply phase.
    let mut all_orphaned_paths: Vec<String> = Vec::new();
    let mut all_orphaned_sessions: Vec<OrphanedSession> = Vec::new();
    // Track stale paths per profile: only clear history when stale in ALL profiles.
    let mut stale_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let profile_count = profiles.len();

    for (name, claude_home) in &profiles {
        if multi_profile {
            println!("--- profile: {} ({}) ---", name, tilde_path(&claude_home.to_string_lossy()));
        }

        let projects_dir = claude_home.join("projects");

        // Orphan analysis: filter from the shared JSON analysis.
        let orphaned_paths: Vec<String> = analysis
            .orphaned_projects
            .iter()
            .map(|p| p.path.clone())
            .collect();
        let (sessions, session_dir_total_size) =
            scan_session_folders(&projects_dir, &orphaned_paths);

        // Stale project detection (per-profile session dirs).
        let stale_projects = if let Some(days) = options.stale_days {
            detect_stale_projects(&analysis.parsed_json, &projects_dir, days)
        } else {
            Vec::new()
        };

        // Cache measurement (per-profile claude home).
        let cache_dirs = if options.cache {
            measure_cache_dirs(claude_home)
        } else {
            Vec::new()
        };

        // Accumulate orphaned paths (deduplicated).
        for path in &orphaned_paths {
            if !all_orphaned_paths.contains(path) {
                all_orphaned_paths.push(path.clone());
            }
        }
        // Count how many profiles see each project as stale.
        for stale in &stale_projects {
            *stale_counts.entry(stale.path.clone()).or_insert(0) += 1;
        }

        let report = DietReport {
            orphaned_projects: analysis.orphaned_projects.clone(),
            orphaned_sessions: sessions,
            stale_projects,
            cache_dirs,
            active_project_count: analysis.active_count,
            config_file_size: analysis.config_file_size,
            session_dir_total_size,
            claude_home: claude_home.clone(),
        };

        print_report(&report, false);

        if options.apply {
            // Per-profile cleanup: cache contents and stale session folders.
            if !report.cache_dirs.is_empty() {
                let (cache_removed, cache_freed) = remove_cache_contents(&report.cache_dirs)?;
                println!(
                    "{} Cleared {} cache dirs ({} freed)",
                    "✓".green(),
                    cache_removed.to_string().green(),
                    format_size(cache_freed).cyan()
                );
            }

            // Remove stale session folders (per-profile projects dir).
            for stale in &report.stale_projects {
                let session_name = project_path_to_session_dir(&stale.path);
                let session_path = projects_dir.join(&session_name);
                if session_path.is_dir() {
                    if let Err(e) = fs::remove_dir_all(&session_path) {
                        eprintln!(
                            "{} failed to remove stale session {}: {}",
                            "warning:".yellow().bold(),
                            session_path.display(),
                            e
                        );
                    }
                }
            }

            // Accumulate orphaned sessions for post-loop removal.
            all_orphaned_sessions.extend(report.orphaned_sessions);
        }
    }

    // Post-loop apply: single atomic write for shared ~/.claude.json mutations.
    if options.apply {
        // Only clear history for projects stale in ALL targeted profiles.
        let all_stale_paths: Vec<String> = stale_counts
            .into_iter()
            .filter(|&(_, count)| count >= profile_count)
            .map(|(path, _)| path)
            .collect();

        let has_json_mutations = !all_orphaned_paths.is_empty() || !all_stale_paths.is_empty();

        // Only backup/write if ~/.claude.json exists and there are mutations.
        if has_json_mutations && analysis.parsed_json != serde_json::Value::Null {
            let backup_path = backup_claude_json(&claude_json_path)?;
            println!(
                "{} Backed up to {}",
                "✓".green(),
                tilde_path(&backup_path.to_string_lossy())
            );
            rotate_backups(&claude_json_path, 5)?;

            // In-memory mutations on the shared JSON.
            remove_orphaned_entries(&mut analysis.parsed_json, &all_orphaned_paths);
            if !all_orphaned_paths.is_empty() {
                println!(
                    "{} Removed {} orphaned project entries",
                    "✓".green(),
                    all_orphaned_paths.len().to_string().green()
                );
            }

            clear_stale_history(&mut analysis.parsed_json, &all_stale_paths);
            if !all_stale_paths.is_empty() {
                println!(
                    "{} Cleared history for {} stale projects",
                    "✓".green(),
                    all_stale_paths.len().to_string().green()
                );
            }

            // Single atomic write.
            let serialized =
                serde_json::to_string_pretty(&analysis.parsed_json).map_err(|e| {
                    CyoloError::ConfigIoError {
                        context: format!(
                            "failed to serialize JSON for {}",
                            claude_json_path.display()
                        ),
                        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
                    }
                })?;
            atomic_write_json(&claude_json_path, &serialized)?;
        }

        // Remove orphaned session folders.
        let (removed_count, bytes_freed) = remove_session_folders(&all_orphaned_sessions)?;
        if removed_count > 0 {
            println!(
                "{} Removed {} orphaned session folders ({} freed)",
                "✓".green(),
                removed_count.to_string().green(),
                format_size(bytes_freed).cyan()
            );
        }

        println!("{} Cleanup complete.", "✓".green());
    }

    Ok(())
}

#[cfg(test)]
mod tests;
