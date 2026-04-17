use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::CyoloError;

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
#[derive(Debug)]
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
                            "warning: could not read mtime for {}: {}",
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
                    "warning: cache directory {} is no longer a real directory, skipping",
                    cache.path.display()
                );
                continue;
            }
        }

        let entries = match fs::read_dir(&cache.path) {
            Ok(entries) => entries,
            Err(e) => {
                eprintln!(
                    "warning: could not read cache directory {}: {}",
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
                        "warning: could not read entry in {}: {}",
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
                        "warning: could not stat {}: {}",
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
                    "warning: failed to remove {}: {}",
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

    // Special case: no orphans at all.
    if report.orphaned_projects.is_empty() && report.orphaned_sessions.is_empty() {
        return format!(
            "cyolo diet — analyzing {claude_home_display}\n\n\
             No orphaned projects found. Nothing to clean up.\n"
        );
    }

    let mut buf = String::new();

    // Header
    writeln!(buf, "cyolo diet — analyzing {claude_home_display}").unwrap();
    writeln!(buf).unwrap();

    // ── ~/.claude.json section ──────────────────────────────────
    let claude_json_display = format!("{claude_home_display}.json");
    let orphan_total_size: u64 = report.orphaned_projects.iter().map(|o| o.entry_size).sum();

    writeln!(
        buf,
        "{:<45} {}",
        format!("{claude_json_display}:"),
        format_size(report.config_file_size)
    )
    .unwrap();

    let orphan_count = report.orphaned_projects.len();

    // Orphaned projects line (always shown)
    writeln!(
        buf,
        "  ├─ {:<41} {}  (removable)",
        format!("orphaned projects ({orphan_count}):"),
        format_size(orphan_total_size)
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
                "  │   {item_char} {:<37} {}",
                tilde_path(&orphan.path),
                format_size(orphan.entry_size)
            )
            .unwrap();
        }
        if orphan_count > 5 {
            writeln!(
                buf,
                "  │   └─ ... {} more",
                orphan_count - 5
            )
            .unwrap();
        }
    }

    // Active projects line (always shown)
    let active_size = report.config_file_size.saturating_sub(orphan_total_size);
    writeln!(
        buf,
        "  └─ {:<41} {}  (keep)",
        format!("active projects ({}):", report.active_project_count),
        format_size(active_size)
    )
    .unwrap();

    writeln!(buf).unwrap();

    // ── ~/.claude/projects/ section ─────────────────────────────
    writeln!(
        buf,
        "{:<45} {}",
        format!("{claude_home_display}/projects/:"),
        format_size(report.session_dir_total_size)
    )
    .unwrap();

    let session_total_size: u64 = report.orphaned_sessions.iter().map(|s| s.total_size).sum();
    let session_count = report.orphaned_sessions.len();
    // Always show orphaned session folders line
    writeln!(
        buf,
        "  └─ {:<41} {}  (removable)",
        format!("orphaned session folders ({session_count}):"),
        format_size(session_total_size)
    )
    .unwrap();

    writeln!(buf).unwrap();

    // Total reclaimable
    let total_reclaimable = orphan_total_size + session_total_size;
    writeln!(buf, "Total reclaimable: {}", format_size(total_reclaimable)).unwrap();
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
                "warning: failed to remove old backup {}: {}",
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

/// Remove orphaned project entries from the parsed JSON and write back to disk.
///
/// Uses `serde_json::Map::retain` with a `HashSet` for O(1) lookup.
/// The file is written atomically via `atomic_write_json()` to prevent
/// corruption on crash.
pub(crate) fn remove_orphaned_entries(
    parsed_json: &mut serde_json::Value,
    orphaned_paths: &[String],
    claude_json_path: &Path,
) -> Result<(), CyoloError> {
    let orphaned_set: HashSet<&str> = orphaned_paths.iter().map(|s| s.as_str()).collect();

    if let Some(projects) = parsed_json
        .get_mut("projects")
        .and_then(|v| v.as_object_mut())
    {
        projects.retain(|key, _| !orphaned_set.contains(key.as_str()));
    }

    let serialized =
        serde_json::to_string_pretty(parsed_json).map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to serialize JSON for {}", claude_json_path.display()),
            source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
        })?;

    atomic_write_json(claude_json_path, &serialized)?;

    Ok(())
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

/// Execute the cleanup: backup, remove orphaned entries, remove session folders.
///
/// Prints a summary of actions taken.
pub(crate) fn apply(
    report: &DietReport,
    parsed_json: &mut serde_json::Value,
    claude_json_path: &Path,
) -> Result<(), CyoloError> {
    // Step 1: Backup
    let backup_path = backup_claude_json(claude_json_path)?;
    println!(
        "Backed up to {}",
        tilde_path(&backup_path.to_string_lossy())
    );
    rotate_backups(claude_json_path, 5)?;

    // Step 2: Remove orphaned entries from JSON
    let orphaned_paths: Vec<String> = report
        .orphaned_projects
        .iter()
        .map(|o| o.path.clone())
        .collect();
    remove_orphaned_entries(parsed_json, &orphaned_paths, claude_json_path)?;
    println!(
        "Removed {} orphaned project entries",
        report.orphaned_projects.len()
    );

    // Step 3: Remove orphaned session folders
    let (removed_count, bytes_freed) = remove_session_folders(&report.orphaned_sessions)?;
    println!(
        "Removed {} orphaned session folders ({} freed)",
        removed_count,
        format_size(bytes_freed)
    );

    Ok(())
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
fn parse_diet_args(args: &[String]) -> Result<DietOptions, CyoloError> {
    let mut apply = false;
    let mut force = false;
    let mut stale_days: Option<u32> = None;
    let mut cache = false;
    let mut profile: Option<String> = None;
    let mut all = false;

    let usage = "Usage: cyolo diet [--apply] [--force] [--stale-days <N>] [--cache] [--profile <name>] [--all]";

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--apply" => apply = true,
            "--force" => force = true,
            "--cache" => cache = true,
            "--all" => all = true,
            "--stale-days" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("cyolo: --stale-days requires a value");
                    eprintln!("{usage}");
                    return Err(CyoloError::NonZeroExit(1));
                }
                let n: u32 = args[i].parse().map_err(|_| {
                    eprintln!("cyolo: --stale-days value must be a positive integer, got '{}'", args[i]);
                    eprintln!("{usage}");
                    CyoloError::NonZeroExit(1)
                })?;
                if n == 0 {
                    eprintln!("cyolo: --stale-days value must be greater than zero");
                    eprintln!("{usage}");
                    return Err(CyoloError::NonZeroExit(1));
                }
                stale_days = Some(n);
            }
            "--profile" => {
                i += 1;
                if i >= args.len() || args[i].starts_with("--") {
                    eprintln!("cyolo: --profile requires a value");
                    eprintln!("{usage}");
                    return Err(CyoloError::NonZeroExit(1));
                }
                profile = Some(args[i].clone());
            }
            _ => {
                eprintln!("cyolo: unknown diet option '{}'", args[i]);
                eprintln!("{usage}");
                return Err(CyoloError::NonZeroExit(1));
            }
        }
        i += 1;
    }

    if profile.is_some() && all {
        eprintln!("cyolo: --profile and --all are mutually exclusive");
        eprintln!("{usage}");
        return Err(CyoloError::NonZeroExit(1));
    }

    Ok(DietOptions { apply, force, stale_days, cache, profile, all })
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

/// Entry point for the `diet` subcommand.
///
/// Orchestrates the full pipeline: parse args → analyze → scan → report → apply.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let options = parse_diet_args(args)?;

    if !options.force && is_claude_running() {
        eprintln!("cyolo: Claude is currently running. Stop Claude first, or use --force to proceed.");
        return Err(CyoloError::NonZeroExit(1));
    }

    let (home, claude_home) = resolve_claude_home()?;
    let claude_json_path = home.join(".claude.json");
    let projects_dir = claude_home.join("projects");

    let mut analysis = analyze_claude_json(&claude_json_path)?;

    let orphaned_paths: Vec<String> = analysis
        .orphaned_projects
        .iter()
        .map(|p| p.path.clone())
        .collect();
    let (sessions, session_dir_total_size) =
        scan_session_folders(&projects_dir, &orphaned_paths);

    let report = DietReport {
        orphaned_projects: analysis.orphaned_projects,
        orphaned_sessions: sessions,
        active_project_count: analysis.active_count,
        config_file_size: analysis.config_file_size,
        session_dir_total_size,
        claude_home,
    };

    // Always show the dry-run report first; show "Cleanup complete." only after
    // apply() succeeds so a failed cleanup never prints a success footer.
    print_report(&report, false);

    if options.apply {
        apply(&report, &mut analysis.parsed_json, &claude_json_path)?;
        println!("Cleanup complete.");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper: write a string to a file inside a temp dir.
    fn write_claude_json(dir: &TempDir, content: &str) -> PathBuf {
        let path = dir.path().join("claude.json");
        fs::write(&path, content).unwrap();
        path
    }

    // ── analyze_claude_json tests ────────────────────────────────

    #[test]
    fn test_analyze_missing_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent.json");

        let result = analyze_claude_json(&path).unwrap();

        assert!(result.orphaned_projects.is_empty());
        assert_eq!(result.active_count, 0);
        assert_eq!(result.config_file_size, 0);
        assert_eq!(result.parsed_json, serde_json::Value::Null);
    }

    #[test]
    fn test_analyze_no_projects_key() {
        let dir = TempDir::new().unwrap();
        let path = write_claude_json(&dir, r#"{"version": "1.0"}"#);

        let result = analyze_claude_json(&path).unwrap();

        assert!(result.orphaned_projects.is_empty());
        assert_eq!(result.active_count, 0);
        assert!(result.config_file_size > 0);
        assert!(result.parsed_json.get("version").is_some());
    }

    #[test]
    fn test_analyze_empty_projects() {
        let dir = TempDir::new().unwrap();
        let path = write_claude_json(&dir, r#"{"projects": {}}"#);

        let result = analyze_claude_json(&path).unwrap();

        assert!(result.orphaned_projects.is_empty());
        assert_eq!(result.active_count, 0);
        assert!(result.config_file_size > 0);
    }

    #[test]
    fn test_analyze_detects_orphans() {
        let dir = TempDir::new().unwrap();
        let content = r#"{"projects": {
            "/nonexistent/path/alpha": {"name": "alpha"},
            "/nonexistent/path/beta": {"name": "beta"}
        }}"#;
        let path = write_claude_json(&dir, content);

        let result = analyze_claude_json(&path).unwrap();

        assert_eq!(result.orphaned_projects.len(), 2);
        assert_eq!(result.active_count, 0);
        let orphan_paths: Vec<&str> = result.orphaned_projects.iter().map(|o| o.path.as_str()).collect();
        assert!(orphan_paths.contains(&"/nonexistent/path/alpha"));
        assert!(orphan_paths.contains(&"/nonexistent/path/beta"));
        // Each entry should have non-zero size
        for orphan in &result.orphaned_projects {
            assert!(orphan.entry_size > 0);
        }
    }

    #[test]
    fn test_analyze_mixed() {
        // Create a real directory that "exists"
        let real_dir = TempDir::new().unwrap();
        let real_path = real_dir.path().to_string_lossy().to_string();

        let dir = TempDir::new().unwrap();
        let content = format!(
            r#"{{"projects": {{
                "{}": {{"name": "active"}},
                "/nonexistent/orphan/one": {{"name": "orphan1"}},
                "/nonexistent/orphan/two": {{"name": "orphan2"}}
            }}}}"#,
            real_path
        );
        let path = write_claude_json(&dir, &content);

        let result = analyze_claude_json(&path).unwrap();

        assert_eq!(result.orphaned_projects.len(), 2);
        assert_eq!(result.active_count, 1);
        assert!(result.config_file_size > 0);
        // parsed_json should have the projects key
        assert!(result.parsed_json.get("projects").is_some());
    }

    // ── format_size tests ────────────────────────────────────────

    #[test]
    fn test_format_size_zero() {
        assert_eq!(format_size(0), "0 B");
    }

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(512), "512 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1_500_000), "1.4 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1_500_000_000), "1.4 GB");
    }

    #[test]
    fn test_format_size_exact_boundary() {
        assert_eq!(format_size(1024), "1.0 KB");
    }

    #[test]
    fn test_format_size_just_below_mb() {
        // 1024*1024 - 1 = 1_048_575 should promote to MB, not show "1024.0 KB"
        assert_eq!(format_size(1_048_575), "1.0 MB");
    }

    #[test]
    fn test_format_size_exact_mb() {
        assert_eq!(format_size(1_048_576), "1.0 MB");
    }

    #[test]
    fn test_format_size_just_below_gb() {
        // 1024^3 - 1 = 1_073_741_823 should promote to GB, not show "1024.0 MB"
        assert_eq!(format_size(1_073_741_823), "1.0 GB");
    }

    #[test]
    fn test_format_size_exact_gb() {
        assert_eq!(format_size(1_073_741_824), "1.0 GB");
    }

    // ── scan_session_folders tests ──────────────────────────────

    #[test]
    fn test_path_encoding() {
        assert_eq!(
            project_path_to_session_dir("/Users/codingmax/Private/labs/test-bot"),
            "-Users-codingmax-Private-labs-test-bot"
        );
    }

    #[test]
    fn test_path_encoding_root() {
        assert_eq!(project_path_to_session_dir("/"), "-");
    }

    #[test]
    fn test_path_encoding_nested() {
        assert_eq!(
            project_path_to_session_dir("/a/b/c/d"),
            "-a-b-c-d"
        );
    }

    #[test]
    fn test_dir_size_nonexistent() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("nonexistent");
        assert_eq!(dir_size(&path), 0);
    }

    #[test]
    fn test_dir_size_empty() {
        let dir = TempDir::new().unwrap();
        assert_eq!(dir_size(dir.path()), 0);
    }

    #[test]
    fn test_dir_size_with_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.txt"), "hello").unwrap(); // 5 bytes
        fs::write(dir.path().join("b.txt"), "world!").unwrap(); // 6 bytes
        assert_eq!(dir_size(dir.path()), 11);
    }

    #[test]
    fn test_dir_size_recursive() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join("sub")).unwrap();
        fs::write(dir.path().join("a.txt"), "hi").unwrap(); // 2 bytes
        fs::write(dir.path().join("sub").join("b.txt"), "hey").unwrap(); // 3 bytes
        assert_eq!(dir_size(dir.path()), 5);
    }

    #[test]
    fn test_scan_missing_projects_dir() {
        let dir = TempDir::new().unwrap();
        let missing = dir.path().join("nonexistent");
        let orphaned_paths = vec!["/some/path".to_string()];
        let (sessions, total) = scan_session_folders(&missing, &orphaned_paths);
        assert!(sessions.is_empty());
        assert_eq!(total, 0);
    }

    #[test]
    fn test_scan_finds_orphaned_sessions() {
        let dir = TempDir::new().unwrap();
        let projects_dir = dir.path();

        // Create session folders matching orphaned paths
        let session_name =
            project_path_to_session_dir("/Users/codingmax/Private/labs/test-bot");
        let session_path = projects_dir.join(&session_name);
        fs::create_dir(&session_path).unwrap();
        fs::write(session_path.join("data.json"), "test data here").unwrap(); // 14 bytes

        // Also create a non-orphaned session folder
        let active_name =
            project_path_to_session_dir("/Users/codingmax/active-project");
        let active_path = projects_dir.join(&active_name);
        fs::create_dir(&active_path).unwrap();
        fs::write(active_path.join("state.json"), "active").unwrap(); // 6 bytes

        let orphaned_paths =
            vec!["/Users/codingmax/Private/labs/test-bot".to_string()];
        let (sessions, total) = scan_session_folders(projects_dir, &orphaned_paths);

        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].folder_path, session_path);
        assert_eq!(sessions[0].total_size, 14);
        // total should include ALL session dirs (orphaned + active)
        assert_eq!(total, 20); // 14 + 6
    }

    #[test]
    fn test_scan_no_matching_sessions() {
        let dir = TempDir::new().unwrap();
        let projects_dir = dir.path();

        // Create a session folder that does NOT match any orphaned path
        let other_name = project_path_to_session_dir("/some/other/project");
        fs::create_dir(projects_dir.join(&other_name)).unwrap();

        let orphaned_paths = vec!["/Users/nonexistent/path".to_string()];
        let (sessions, total) = scan_session_folders(projects_dir, &orphaned_paths);

        assert!(sessions.is_empty());
        // total should still count the existing session folder
        assert_eq!(total, 0); // empty dir has size 0
    }

    #[test]
    fn test_dir_size_ignores_symlinks() {
        let dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create a real file in the target (should NOT be counted)
        fs::write(target_dir.path().join("big.bin"), vec![0u8; 1000]).unwrap();

        // Create a symlink inside the scanned dir pointing to the target
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target_dir.path(), dir.path().join("link")).unwrap();
        }

        // Create a real file alongside the symlink
        fs::write(dir.path().join("real.txt"), "data").unwrap(); // 4 bytes

        // dir_size should only count the real file, NOT follow the symlink
        assert_eq!(dir_size(dir.path()), 4);
    }

    #[test]
    fn test_scan_ignores_file_as_session() {
        let dir = TempDir::new().unwrap();
        let projects_dir = dir.path();

        // Create a regular file (not a directory) with the encoded session name
        let session_name = project_path_to_session_dir("/Users/codingmax/fake-project");
        fs::write(projects_dir.join(&session_name), "not a dir").unwrap();

        let orphaned_paths = vec!["/Users/codingmax/fake-project".to_string()];
        let (sessions, _total) = scan_session_folders(projects_dir, &orphaned_paths);

        // Should NOT be included because it's a file, not a directory
        assert!(sessions.is_empty());
    }

    // ── build_report_string tests ──────────────────────────────

    /// Helper: create a DietReport with the given orphaned projects and sessions.
    fn make_report(
        orphan_paths: &[(&str, u64)],
        sessions: Vec<OrphanedSession>,
        active_count: usize,
    ) -> DietReport {
        let orphaned_projects = orphan_paths
            .iter()
            .map(|(p, s)| OrphanedProject {
                path: p.to_string(),
                entry_size: *s,
            })
            .collect();
        let session_total: u64 = sessions.iter().map(|s| s.total_size).sum();
        DietReport {
            orphaned_projects,
            orphaned_sessions: sessions,
            active_project_count: active_count,
            config_file_size: 50_000,
            session_dir_total_size: session_total + 100_000, // sessions + active
            claude_home: PathBuf::from("/fakehome/.claude"),
        }
    }

    #[test]
    fn test_report_no_orphans() {
        let report = make_report(&[], vec![], 3);
        let output = build_report_string(&report, false);
        assert!(output.contains("No orphaned projects found. Nothing to clean up."));
        assert!(output.contains("cyolo diet — analyzing /fakehome/.claude"));
        // Should NOT contain tree structure or footer
        assert!(!output.contains("├─"));
        assert!(!output.contains("Run with --apply"));
    }

    #[test]
    fn test_report_one_orphan() {
        let report = make_report(
            &[("/fakehome/projects/deleted", 1024)],
            vec![],
            2,
        );
        let output = build_report_string(&report, false);
        assert!(output.contains("orphaned projects (1):"));
        assert!(output.contains("/fakehome/projects/deleted"));
        assert!(output.contains("1.0 KB"));
        assert!(output.contains("active projects (2):"));
        // Active size = config_file_size(50000) - orphan_size(1024) = 48976
        assert!(output.contains("47.8 KB"));
        assert!(output.contains("(keep)"));
        assert!(output.contains("Run with --apply to proceed."));
    }

    #[test]
    fn test_report_five_orphans() {
        let paths: Vec<(&str, u64)> = (1..=5)
            .map(|i| {
                // Use a static slice for the path strings
                let path: &str = match i {
                    1 => "/fakehome/projects/p1",
                    2 => "/fakehome/projects/p2",
                    3 => "/fakehome/projects/p3",
                    4 => "/fakehome/projects/p4",
                    5 => "/fakehome/projects/p5",
                    _ => unreachable!(),
                };
                (path, 500u64)
            })
            .collect();
        let report = make_report(&paths, vec![], 1);
        let output = build_report_string(&report, false);
        assert!(output.contains("orphaned projects (5):"));
        // All 5 should be listed
        for i in 1..=5 {
            assert!(output.contains(&format!("/fakehome/projects/p{i}")));
        }
        // No "... more" line
        assert!(!output.contains("more"));
    }

    #[test]
    fn test_report_ten_orphans() {
        let path_strings: Vec<String> = (1..=10)
            .map(|i| format!("/fakehome/projects/p{i:02}"))
            .collect();
        let paths: Vec<(&str, u64)> = path_strings
            .iter()
            .map(|s| (s.as_str(), 200u64))
            .collect();
        let report = make_report(&paths, vec![], 0);
        let output = build_report_string(&report, false);
        assert!(output.contains("orphaned projects (10):"));
        // First 5 listed
        for i in 1..=5 {
            assert!(output.contains(&format!("/fakehome/projects/p{i:02}")));
        }
        // 6-10 collapsed
        assert!(output.contains("... 5 more"));
        // Path 6 should NOT be listed individually
        assert!(!output.contains("/fakehome/projects/p06"));
    }

    #[test]
    fn test_report_applied_footer() {
        let report = make_report(
            &[("/fakehome/projects/x", 512)],
            vec![],
            0,
        );
        let output = build_report_string(&report, true);
        assert!(output.contains("Cleanup complete."));
        assert!(!output.contains("Run with --apply"));
    }

    #[test]
    fn test_report_dry_run_footer() {
        let report = make_report(
            &[("/fakehome/projects/x", 512)],
            vec![],
            0,
        );
        let output = build_report_string(&report, false);
        assert!(output.contains("Run with --apply to proceed."));
        assert!(!output.contains("Cleanup complete."));
    }

    #[test]
    fn test_report_with_orphaned_sessions() {
        let sessions = vec![OrphanedSession {
            folder_path: PathBuf::from("/fakehome/.claude/projects/-fakehome-projects-x"),
            total_size: 5000,
        }];
        let report = make_report(
            &[("/fakehome/projects/x", 512)],
            sessions,
            1,
        );
        let output = build_report_string(&report, false);
        assert!(output.contains("orphaned session folders (1):"));
        assert!(output.contains("Total reclaimable:"));
    }

    // ── format_timestamp tests ─────────────────────────────────

    #[test]
    fn test_format_timestamp_epoch() {
        // Unix epoch: 1970-01-01 00:00:00
        assert_eq!(format_timestamp(0), "19700101000000");
    }

    #[test]
    fn test_format_timestamp_known_date() {
        // 1700000000 = 2023-11-14 22:13:20 UTC
        assert_eq!(format_timestamp(1_700_000_000), "20231114221320");
    }

    #[test]
    fn test_format_timestamp_leap_year() {
        // 2024-02-29 12:00:00 UTC = 1709208000
        assert_eq!(format_timestamp(1_709_208_000), "20240229120000");
    }

    #[test]
    fn test_format_timestamp_year_2000() {
        // 2000-01-01 00:00:00 UTC = 946684800
        assert_eq!(format_timestamp(946_684_800), "20000101000000");
    }

    #[test]
    fn test_format_timestamp_end_of_day() {
        // 2026-04-17 23:59:59 UTC = 1776499199
        // Let's use a known: 2025-12-31 23:59:59 UTC = 1767225599
        assert_eq!(format_timestamp(1_767_225_599), "20251231235959");
    }

    // ── backup_claude_json tests ───────────────────────────────

    #[test]
    fn test_backup_creates_file() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("claude.json");
        fs::write(&json_path, r#"{"projects": {}}"#).unwrap();

        let backup_path = backup_claude_json(&json_path).unwrap();

        assert!(backup_path.exists());
        assert!(backup_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("claude.json.backup-"));
        // Backup content matches original
        let original = fs::read_to_string(&json_path).unwrap();
        let backup = fs::read_to_string(&backup_path).unwrap();
        assert_eq!(original, backup);
    }

    #[test]
    fn test_backup_nonexistent_file_fails() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("nonexistent.json");

        let result = backup_claude_json(&json_path);
        assert!(result.is_err());
    }

    // ── remove_orphaned_entries tests ──────────────────────────

    #[test]
    fn test_remove_entries_preserves_other_fields() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("claude.json");
        let content = r#"{
  "version": "1.0",
  "projects": {
    "/active/project": {"name": "active"},
    "/orphan/one": {"name": "orphan1"},
    "/orphan/two": {"name": "orphan2"}
  },
  "settings": {"theme": "dark"}
}"#;
        fs::write(&json_path, content).unwrap();

        let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
        let orphaned = vec!["/orphan/one".to_string(), "/orphan/two".to_string()];

        remove_orphaned_entries(&mut parsed, &orphaned, &json_path).unwrap();

        // Verify on-disk file
        let written: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&json_path).unwrap()).unwrap();
        // Orphans removed
        let projects = written["projects"].as_object().unwrap();
        assert_eq!(projects.len(), 1);
        assert!(projects.contains_key("/active/project"));
        assert!(!projects.contains_key("/orphan/one"));
        assert!(!projects.contains_key("/orphan/two"));
        // Other top-level fields preserved
        assert_eq!(written["version"], "1.0");
        assert_eq!(written["settings"]["theme"], "dark");
    }

    #[test]
    fn test_remove_entries_no_projects_key() {
        let dir = TempDir::new().unwrap();
        let json_path = dir.path().join("claude.json");
        let content = r#"{"version": "1.0"}"#;
        fs::write(&json_path, content).unwrap();

        let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
        let orphaned = vec!["/orphan/one".to_string()];

        // Should succeed without error (no-op)
        remove_orphaned_entries(&mut parsed, &orphaned, &json_path).unwrap();
    }

    // ── atomic_write_json tests ─────────────────────────────────

    #[test]
    fn test_atomic_write_json_basic() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.json");
        let content = r#"{"key": "value"}"#;

        atomic_write_json(&path, content).unwrap();

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, content);
    }

    #[test]
    fn test_atomic_write_json_no_leftover_tmp() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.json");
        let tmp_path = path.with_extension("json.tmp");

        atomic_write_json(&path, "test content").unwrap();

        // The .json.tmp file should not remain after a successful write
        assert!(!tmp_path.exists());
        assert!(path.exists());
    }

    #[test]
    fn test_atomic_write_json_overwrites_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("output.json");

        // Write initial content
        fs::write(&path, "old content").unwrap();

        // Overwrite with atomic_write_json
        atomic_write_json(&path, "new content").unwrap();

        let written = fs::read_to_string(&path).unwrap();
        assert_eq!(written, "new content");
    }

    // ── remove_session_folders tests ───────────────────────────

    #[test]
    fn test_remove_session_folders_regular() {
        let dir = TempDir::new().unwrap();
        let session_dir = dir.path().join("session1");
        fs::create_dir(&session_dir).unwrap();
        fs::write(session_dir.join("data.json"), "test data").unwrap(); // 9 bytes
        fs::create_dir(session_dir.join("sub")).unwrap();
        fs::write(session_dir.join("sub").join("nested.txt"), "nested").unwrap(); // 6 bytes

        let sessions = vec![OrphanedSession {
            folder_path: session_dir.clone(),
            total_size: 15,
        }];

        let (removed, freed) = remove_session_folders(&sessions).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 15);
        assert!(!session_dir.exists());
    }

    #[test]
    fn test_remove_session_folders_with_symlinks() {
        let dir = TempDir::new().unwrap();
        let target_dir = TempDir::new().unwrap();

        // Create a target file that should NOT be deleted
        let target_file = target_dir.path().join("important.txt");
        fs::write(&target_file, "important data").unwrap();

        // Create session folder with a symlink and a regular file
        let session_dir = dir.path().join("session-with-symlink");
        fs::create_dir(&session_dir).unwrap();
        fs::write(session_dir.join("regular.txt"), "regular").unwrap();

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(&target_file, session_dir.join("link.txt")).unwrap();
        }

        let sessions = vec![OrphanedSession {
            folder_path: session_dir.clone(),
            total_size: 100,
        }];

        let (removed, freed) = remove_session_folders(&sessions).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 100);
        assert!(!session_dir.exists());
        // Target file should still exist — symlink was unlinked, not followed
        assert!(target_file.exists());
        assert_eq!(fs::read_to_string(&target_file).unwrap(), "important data");
    }

    #[test]
    fn test_remove_session_folders_empty_list() {
        let (removed, freed) = remove_session_folders(&[]).unwrap();
        assert_eq!(removed, 0);
        assert_eq!(freed, 0);
    }

    #[test]
    fn test_remove_session_folder_symlinked_root() {
        // Regression: if the session folder itself is a symlink to an external
        // directory, we must unlink the symlink — never read_dir or remove_dir_all
        // through it, which would mutate data outside ~/.claude/projects/.
        let dir = TempDir::new().unwrap();
        let external_dir = TempDir::new().unwrap();

        // Create files inside the external directory
        let external_file = external_dir.path().join("precious.txt");
        fs::write(&external_file, "do not delete").unwrap();

        // Create a symlink session folder pointing to the external dir
        let session_link = dir.path().join("session-symlink");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(external_dir.path(), &session_link).unwrap();
        }

        let sessions = vec![OrphanedSession {
            folder_path: session_link.clone(),
            total_size: 50,
        }];

        let (removed, freed) = remove_session_folders(&sessions).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 50);
        // The symlink itself should be gone
        assert!(!session_link.exists());
        // The external directory and its contents must be untouched
        assert!(external_dir.path().exists());
        assert!(external_file.exists());
        assert_eq!(fs::read_to_string(&external_file).unwrap(), "do not delete");
    }

    #[test]
    fn test_scan_skips_symlinked_session_root() {
        // If a session folder entry is a symlink, scan_session_folders should skip it.
        let dir = TempDir::new().unwrap();
        let external_dir = TempDir::new().unwrap();
        fs::write(external_dir.path().join("data.txt"), "external").unwrap();

        let session_name = project_path_to_session_dir("/Users/codingmax/symlinked-proj");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(external_dir.path(), dir.path().join(&session_name))
                .unwrap();
        }

        let orphaned_paths = vec!["/Users/codingmax/symlinked-proj".to_string()];
        let (sessions, _total) = scan_session_folders(dir.path(), &orphaned_paths);

        // The symlinked session folder should NOT be included
        assert!(sessions.is_empty());
    }

    // ── parse_diet_args tests ─────────────────────────────────

    #[test]
    fn test_parse_args_empty() {
        let result = parse_diet_args(&[]).unwrap();
        assert!(!result.apply);
        assert!(!result.force);
        assert!(result.stale_days.is_none());
        assert!(!result.cache);
        assert!(result.profile.is_none());
        assert!(!result.all);
    }

    #[test]
    fn test_parse_args_apply() {
        let args = vec!["--apply".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.apply);
        assert!(!result.force);
    }

    #[test]
    fn test_parse_args_unknown_flag() {
        let args = vec!["--verbose".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_force() {
        let args = vec!["--force".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(!result.apply);
        assert!(result.force);
    }

    #[test]
    fn test_parse_args_apply_and_force() {
        let args = vec!["--apply".to_string(), "--force".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.apply);
        assert!(result.force);
    }

    #[test]
    fn test_parse_args_force_and_apply() {
        let args = vec!["--force".to_string(), "--apply".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.apply);
        assert!(result.force);
    }

    #[test]
    fn test_parse_args_unknown_positional() {
        let args = vec!["cleanup".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_unknown_before_apply() {
        // First unknown arg should cause immediate error, even if --apply follows
        let args = vec!["--verbose".to_string(), "--apply".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_apply_then_unknown() {
        // --apply followed by unknown should still fail (validate all args)
        let args = vec!["--apply".to_string(), "--unknown".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_stale_days() {
        let args = vec!["--stale-days".to_string(), "90".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert_eq!(result.stale_days, Some(90));
    }

    #[test]
    fn test_parse_args_stale_days_missing_value() {
        let args = vec!["--stale-days".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_stale_days_zero() {
        let args = vec!["--stale-days".to_string(), "0".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_stale_days_negative() {
        let args = vec!["--stale-days".to_string(), "-1".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_stale_days_non_numeric() {
        let args = vec!["--stale-days".to_string(), "abc".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_cache() {
        let args = vec!["--cache".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.cache);
    }

    #[test]
    fn test_parse_args_profile() {
        let args = vec!["--profile".to_string(), "work".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert_eq!(result.profile, Some("work".to_string()));
    }

    #[test]
    fn test_parse_args_profile_missing_value() {
        let args = vec!["--profile".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_all() {
        let args = vec!["--all".to_string()];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.all);
    }

    #[test]
    fn test_parse_args_profile_and_all_exclusive() {
        let args = vec!["--profile".to_string(), "x".to_string(), "--all".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_profile_swallows_flag() {
        // Regression: --profile --apply should error, not set profile="--apply"
        let args = vec!["--profile".to_string(), "--apply".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_profile_swallows_all_flag() {
        let args = vec!["--profile".to_string(), "--all".to_string()];
        let result = parse_diet_args(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_args_all_flags_combined() {
        let args = vec![
            "--apply".to_string(),
            "--force".to_string(),
            "--stale-days".to_string(),
            "30".to_string(),
            "--cache".to_string(),
            "--all".to_string(),
        ];
        let result = parse_diet_args(&args).unwrap();
        assert!(result.apply);
        assert!(result.force);
        assert_eq!(result.stale_days, Some(30));
        assert!(result.cache);
        assert!(result.profile.is_none());
        assert!(result.all);
    }

    // ── is_claude_running tests ─────────────────────────────

    #[test]
    fn test_is_claude_running_returns_bool() {
        // Verify the function compiles and returns a bool without panicking.
        // Cannot deterministically test true/false since pgrep depends on runtime state.
        let _result: bool = is_claude_running();
    }

    // ── dispatch error tests ──────────────────────────────────

    #[test]
    fn test_dispatch_unknown_arg_returns_error() {
        let args = vec!["--unknown".to_string()];
        let result = dispatch(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_remove_multiple_sessions() {
        let dir = TempDir::new().unwrap();
        let s1 = dir.path().join("s1");
        let s2 = dir.path().join("s2");
        fs::create_dir(&s1).unwrap();
        fs::create_dir(&s2).unwrap();
        fs::write(s1.join("a.txt"), "aaa").unwrap();
        fs::write(s2.join("b.txt"), "bb").unwrap();

        let sessions = vec![
            OrphanedSession {
                folder_path: s1.clone(),
                total_size: 3,
            },
            OrphanedSession {
                folder_path: s2.clone(),
                total_size: 2,
            },
        ];

        let (removed, freed) = remove_session_folders(&sessions).unwrap();

        assert_eq!(removed, 2);
        assert_eq!(freed, 5);
        assert!(!s1.exists());
        assert!(!s2.exists());
    }

    // ── rotate_backups tests ─────────────────────────────────

    #[test]
    fn test_rotate_backups_deletes_oldest() {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join("claude.json");
        fs::write(&base, "original").unwrap();

        let timestamps = [
            "20260417120001",
            "20260417120002",
            "20260417120003",
            "20260417120004",
            "20260417120005",
            "20260417120006",
            "20260417120007",
        ];
        for ts in &timestamps {
            fs::write(dir.path().join(format!("claude.json.backup-{ts}")), "backup").unwrap();
        }

        rotate_backups(&base, 5).unwrap();

        // Oldest 2 should be deleted
        assert!(!dir.path().join("claude.json.backup-20260417120001").exists());
        assert!(!dir.path().join("claude.json.backup-20260417120002").exists());
        // Newest 5 should remain
        for ts in &timestamps[2..] {
            assert!(dir.path().join(format!("claude.json.backup-{ts}")).exists());
        }
        // Base file untouched
        assert_eq!(fs::read_to_string(&base).unwrap(), "original");
    }

    #[test]
    fn test_rotate_backups_keeps_all_when_under_limit() {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join("claude.json");
        fs::write(&base, "original").unwrap();

        for ts in &["20260417120001", "20260417120002", "20260417120003"] {
            fs::write(dir.path().join(format!("claude.json.backup-{ts}")), "backup").unwrap();
        }

        rotate_backups(&base, 5).unwrap();

        // All 3 should still exist
        for ts in &["20260417120001", "20260417120002", "20260417120003"] {
            assert!(dir.path().join(format!("claude.json.backup-{ts}")).exists());
        }
    }

    #[test]
    fn test_rotate_backups_exactly_at_limit() {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join("claude.json");
        fs::write(&base, "original").unwrap();

        for i in 1..=5 {
            fs::write(dir.path().join(format!("claude.json.backup-2026041712000{i}")), "backup").unwrap();
        }

        rotate_backups(&base, 5).unwrap();

        // All 5 should still exist
        for i in 1..=5 {
            assert!(dir.path().join(format!("claude.json.backup-2026041712000{i}")).exists());
        }
    }

    #[test]
    fn test_rotate_backups_no_backups() {
        let dir = TempDir::new().unwrap();
        let base = dir.path().join("claude.json");
        fs::write(&base, "original").unwrap();

        // No backup files exist — should succeed silently
        rotate_backups(&base, 5).unwrap();
    }

    // ── detect_stale_projects tests ────────────────────────────────

    #[test]
    fn test_detect_stale_old_files() {
        use std::fs::FileTimes;
        use std::time::Duration;

        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        // Create session dir with a file that has an old mtime.
        let session_name = project_path_to_session_dir(&project_path);
        let session_path = projects_dir.path().join(&session_name);
        fs::create_dir_all(&session_path).unwrap();

        let file_path = session_path.join("session.jsonl");
        fs::write(&file_path, "some session data").unwrap();

        let stale_days: u32 = 30;
        let old_time = SystemTime::now() - Duration::from_secs(stale_days as u64 * 86400 + 1);
        let file = fs::File::options().write(true).open(&file_path).unwrap();
        file.set_times(FileTimes::new().set_modified(old_time)).unwrap();

        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": []}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), stale_days);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].path, project_path);
        assert!(result[0].last_activity_secs >= stale_days as u64 * 86400);
        assert!(result[0].session_size > 0);
    }

    #[test]
    fn test_detect_stale_recent_files() {
        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        // Create session dir with a recently-modified file (default mtime = now).
        let session_name = project_path_to_session_dir(&project_path);
        let session_path = projects_dir.path().join(&session_name);
        fs::create_dir_all(&session_path).unwrap();
        fs::write(session_path.join("session.jsonl"), "recent data").unwrap();

        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": []}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), 30);

        assert!(result.is_empty(), "recently modified project should not be stale");
    }

    #[test]
    fn test_detect_stale_no_session_dir() {
        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        // No session directory created for this project.
        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": []}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), 30);

        assert!(result.is_empty(), "project with no session dir should not be stale");
    }

    #[test]
    fn test_detect_stale_empty_session_dir() {
        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        // Create an empty session directory.
        let session_name = project_path_to_session_dir(&project_path);
        let session_path = projects_dir.path().join(&session_name);
        fs::create_dir_all(&session_path).unwrap();

        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": []}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), 30);

        assert!(result.is_empty(), "project with empty session dir should not be stale");
    }

    #[test]
    fn test_detect_stale_orphan_skipped() {
        let projects_dir = TempDir::new().unwrap();

        // Project path does NOT exist on disk.
        let nonexistent_path = "/nonexistent/path/to/project";

        let json: serde_json::Value = serde_json::json!({
            "projects": {
                nonexistent_path: {"history": []}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), 30);

        assert!(result.is_empty(), "nonexistent project path should be skipped entirely");
    }

    #[test]
    fn test_detect_stale_history_size() {
        use std::fs::FileTimes;
        use std::time::Duration;

        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        // Create session dir with an old file.
        let session_name = project_path_to_session_dir(&project_path);
        let session_path = projects_dir.path().join(&session_name);
        fs::create_dir_all(&session_path).unwrap();

        let file_path = session_path.join("session.jsonl");
        fs::write(&file_path, "data").unwrap();

        let stale_days: u32 = 7;
        let old_time = SystemTime::now() - Duration::from_secs(stale_days as u64 * 86400 + 3600);
        let file = fs::File::options().write(true).open(&file_path).unwrap();
        file.set_times(FileTimes::new().set_modified(old_time)).unwrap();

        let history_array = serde_json::json!(["cmd1", "cmd2", "cmd3"]);
        let expected_history_size = serde_json::to_string(&history_array).unwrap().len() as u64;

        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": history_array}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), stale_days);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].history_size, expected_history_size);
    }

    #[test]
    fn test_detect_stale_history_non_array_ignored() {
        use std::fs::FileTimes;
        use std::time::Duration;

        let projects_dir = TempDir::new().unwrap();
        let project_dir = TempDir::new().unwrap();
        let project_path = project_dir.path().to_string_lossy().to_string();

        let session_name = project_path_to_session_dir(&project_path);
        let session_path = projects_dir.path().join(&session_name);
        fs::create_dir_all(&session_path).unwrap();

        let file_path = session_path.join("session.jsonl");
        fs::write(&file_path, "data").unwrap();

        let stale_days: u32 = 7;
        let old_time = SystemTime::now() - Duration::from_secs(stale_days as u64 * 86400 + 3600);
        let file = fs::File::options().write(true).open(&file_path).unwrap();
        file.set_times(FileTimes::new().set_modified(old_time)).unwrap();

        // history is a string, not an array — should yield history_size = 0
        let json: serde_json::Value = serde_json::json!({
            "projects": {
                project_path.clone(): {"history": "not-an-array"}
            }
        });

        let result = detect_stale_projects(&json, projects_dir.path(), stale_days);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].history_size, 0, "non-array history should yield history_size=0");
    }

    // ── measure_cache_dirs tests ──────────────────────────────

    #[test]
    fn test_measure_cache_all_exist() {
        let dir = TempDir::new().unwrap();
        let claude_home = dir.path();

        // Create all three cache directories with files
        fs::create_dir(claude_home.join("statsig")).unwrap();
        fs::write(claude_home.join("statsig").join("data.json"), "statsig data").unwrap(); // 12 bytes

        fs::create_dir(claude_home.join("shell-snapshots")).unwrap();
        fs::write(claude_home.join("shell-snapshots").join("snap.bin"), "snapshot").unwrap(); // 8 bytes

        fs::create_dir(claude_home.join("file-history")).unwrap();
        fs::write(claude_home.join("file-history").join("history.log"), "hist").unwrap(); // 4 bytes

        let result = measure_cache_dirs(claude_home);

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].name, "statsig");
        assert_eq!(result[0].size, 12);
        assert_eq!(result[1].name, "shell-snapshots");
        assert_eq!(result[1].size, 8);
        assert_eq!(result[2].name, "file-history");
        assert_eq!(result[2].size, 4);
    }

    #[test]
    fn test_measure_cache_partial() {
        let dir = TempDir::new().unwrap();
        let claude_home = dir.path();

        // Only create statsig
        fs::create_dir(claude_home.join("statsig")).unwrap();
        fs::write(claude_home.join("statsig").join("data.json"), "abc").unwrap(); // 3 bytes

        let result = measure_cache_dirs(claude_home);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "statsig");
        assert_eq!(result[0].size, 3);
    }

    #[test]
    fn test_measure_cache_none_exist() {
        let dir = TempDir::new().unwrap();
        let result = measure_cache_dirs(dir.path());
        assert!(result.is_empty());
    }

    #[test]
    fn test_measure_cache_sizes_match_dir_size() {
        let dir = TempDir::new().unwrap();
        let claude_home = dir.path();

        fs::create_dir(claude_home.join("statsig")).unwrap();
        fs::write(claude_home.join("statsig").join("a.bin"), vec![0u8; 100]).unwrap();
        fs::create_dir(claude_home.join("statsig").join("sub")).unwrap();
        fs::write(
            claude_home.join("statsig").join("sub").join("b.bin"),
            vec![0u8; 50],
        )
        .unwrap();

        let result = measure_cache_dirs(claude_home);

        assert_eq!(result.len(), 1);
        let expected_size = dir_size(&claude_home.join("statsig"));
        assert_eq!(result[0].size, expected_size);
        assert_eq!(result[0].size, 150);
    }

    #[test]
    fn test_measure_cache_ignores_symlink_as_dir() {
        let dir = TempDir::new().unwrap();
        let claude_home = dir.path();
        let target_dir = TempDir::new().unwrap();

        // Create a symlink named "statsig" → should be skipped
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(target_dir.path(), claude_home.join("statsig")).unwrap();
        }

        let result = measure_cache_dirs(claude_home);

        assert!(result.is_empty(), "symlinked cache dir should be skipped");
    }

    #[test]
    fn test_measure_cache_ignores_file_as_dir() {
        let dir = TempDir::new().unwrap();
        let claude_home = dir.path();

        // Create a regular file named "statsig" instead of a directory
        fs::write(claude_home.join("statsig"), "not a dir").unwrap();

        let result = measure_cache_dirs(claude_home);

        assert!(result.is_empty(), "regular file with cache name should be skipped");
    }

    // ── remove_cache_contents tests ───────────────────────────

    #[test]
    fn test_remove_cache_contents_basic() {
        let dir = TempDir::new().unwrap();
        let cache_path = dir.path().join("statsig");
        fs::create_dir(&cache_path).unwrap();
        fs::write(cache_path.join("file1.json"), "data1").unwrap();
        fs::write(cache_path.join("file2.json"), "data2").unwrap();

        let caches = vec![CacheDir {
            name: "statsig".to_string(),
            path: cache_path.clone(),
            size: 10,
        }];

        let (removed, freed) = remove_cache_contents(&caches).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 10);
        // Cache directory itself should still exist
        assert!(cache_path.exists());
        // But should be empty
        let entries: Vec<_> = fs::read_dir(&cache_path).unwrap().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_remove_cache_contents_empty() {
        let dir = TempDir::new().unwrap();
        let cache_path = dir.path().join("empty-cache");
        fs::create_dir(&cache_path).unwrap();

        let caches = vec![CacheDir {
            name: "empty-cache".to_string(),
            path: cache_path.clone(),
            size: 0,
        }];

        let (removed, freed) = remove_cache_contents(&caches).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 0);
        assert!(cache_path.exists());
    }

    #[test]
    fn test_remove_cache_contents_with_subdirs() {
        let dir = TempDir::new().unwrap();
        let cache_path = dir.path().join("file-history");
        fs::create_dir(&cache_path).unwrap();
        fs::write(cache_path.join("top.txt"), "top").unwrap();
        fs::create_dir(cache_path.join("nested")).unwrap();
        fs::write(cache_path.join("nested").join("deep.txt"), "deep").unwrap();
        fs::create_dir(cache_path.join("nested").join("deeper")).unwrap();
        fs::write(
            cache_path.join("nested").join("deeper").join("file.bin"),
            "bin",
        )
        .unwrap();

        let caches = vec![CacheDir {
            name: "file-history".to_string(),
            path: cache_path.clone(),
            size: 10,
        }];

        let (removed, freed) = remove_cache_contents(&caches).unwrap();

        assert_eq!(removed, 1);
        assert_eq!(freed, 10);
        assert!(cache_path.exists());
        let entries: Vec<_> = fs::read_dir(&cache_path).unwrap().collect();
        assert!(entries.is_empty());
    }

    #[test]
    fn test_remove_cache_contents_no_caches() {
        let (removed, freed) = remove_cache_contents(&[]).unwrap();
        assert_eq!(removed, 0);
        assert_eq!(freed, 0);
    }

    #[test]
    fn test_remove_cache_contents_multiple() {
        let dir = TempDir::new().unwrap();

        let c1_path = dir.path().join("c1");
        fs::create_dir(&c1_path).unwrap();
        fs::write(c1_path.join("f1.txt"), "aaa").unwrap();

        let c2_path = dir.path().join("c2");
        fs::create_dir(&c2_path).unwrap();
        fs::write(c2_path.join("f2.txt"), "bb").unwrap();

        let caches = vec![
            CacheDir {
                name: "c1".to_string(),
                path: c1_path.clone(),
                size: 3,
            },
            CacheDir {
                name: "c2".to_string(),
                path: c2_path.clone(),
                size: 2,
            },
        ];

        let (removed, freed) = remove_cache_contents(&caches).unwrap();

        assert_eq!(removed, 2);
        assert_eq!(freed, 5);
        assert!(c1_path.exists());
        assert!(c2_path.exists());
        assert!(fs::read_dir(&c1_path).unwrap().count() == 0);
        assert!(fs::read_dir(&c2_path).unwrap().count() == 0);
    }
}
