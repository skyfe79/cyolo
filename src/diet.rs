use std::fs;
use std::path::{Path, PathBuf};

use crate::error::CyoloError;

/// Parsed CLI arguments for the `diet` subcommand.
#[derive(Debug)]
pub(crate) struct DietOptions {
    /// Whether `--apply` was provided (execute cleanup vs dry-run).
    pub apply: bool,
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
    for entry in entries {
        if let Ok(entry) = entry {
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
        for entry in entries {
            if let Ok(entry) = entry {
                total_session_dir_size += dir_size(&entry.path());
            }
        }
    }

    // Find session folders that correspond to orphaned project paths.
    let mut orphaned_sessions = Vec::new();
    for path in orphaned_paths {
        let session_name = project_path_to_session_dir(path);
        let session_path = projects_dir.join(&session_name);
        if session_path.is_dir() {
            let total_size = dir_size(&session_path);
            orphaned_sessions.push(OrphanedSession {
                folder_path: session_path,
                total_size,
            });
        }
    }

    (orphaned_sessions, total_session_dir_size)
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
}
