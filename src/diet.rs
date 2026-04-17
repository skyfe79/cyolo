use std::path::PathBuf;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
