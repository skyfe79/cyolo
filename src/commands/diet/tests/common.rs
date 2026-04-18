//! Shared fixtures for the diet per-test files. Individual `test_*.rs`
//! files pull what they need via `use super::common::*;`.

// Re-export the fixtures that per-test files reach for via
// `use super::common::*;`. Keeping them public here lets individual
// files stay focused on their assertion body without repeating import
// boilerplate.
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::sync::Once;
pub use tempfile::TempDir;

use super::super::{CacheDir, DietReport, OrphanedProject, OrphanedSession, StaleProject};

/// Disable ANSI colors exactly once per test-binary lifetime so assertions
/// against captured stderr / stdout don't see color-escape bytes.
static INIT_COLORS: Once = Once::new();

pub fn setup() {
    INIT_COLORS.call_once(|| owo_colors::set_override(false));
}

/// Write `content` into `<dir>/claude.json` and return the file path.
#[allow(dead_code)]
pub fn write_claude_json(dir: &TempDir, content: &str) -> PathBuf {
    let path = dir.path().join("claude.json");
    std::fs::write(&path, content).unwrap();
    path
}

/// Build a `DietReport` with orphans + sessions only.
#[allow(dead_code)]
pub fn make_report(
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
        stale_projects: Vec::new(),
        cache_dirs: Vec::new(),
        active_project_count: active_count,
        config_file_size: 50_000,
        session_dir_total_size: session_total + 100_000,
        claude_home: PathBuf::from("/fakehome/.claude"),
    }
}

/// Build a `DietReport` with every optional slice populated — used by the
/// tests that exercise the full dry-run output.
#[allow(dead_code)]
pub fn make_report_full(
    orphan_paths: &[(&str, u64)],
    sessions: Vec<OrphanedSession>,
    stale: Vec<StaleProject>,
    caches: Vec<CacheDir>,
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
        stale_projects: stale,
        cache_dirs: caches,
        active_project_count: active_count,
        config_file_size: 50_000,
        session_dir_total_size: session_total + 100_000,
        claude_home: PathBuf::from("/fakehome/.claude"),
    }
}
