use super::*;
use std::fs;
use std::sync::Once;
use tempfile::TempDir;

// Disable ANSI color output once per test binary so tests exercising
// parse_diet_args/dispatch/build_report_string don't emit ANSI codes
// into captured stderr/stdout.
static INIT_COLORS: Once = Once::new();
fn setup() {
    INIT_COLORS.call_once(|| owo_colors::set_override(false));
}

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
        stale_projects: Vec::new(),
        cache_dirs: Vec::new(),
        active_project_count: active_count,
        config_file_size: 50_000,
        session_dir_total_size: session_total + 100_000, // sessions + active
        claude_home: PathBuf::from("/fakehome/.claude"),
    }
}

#[test]
fn test_report_no_orphans() {
    setup();
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
    setup();
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
    setup();
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
    setup();
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
    setup();
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
    setup();
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
    setup();
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
    let content = r#"{
  "version": "1.0",
  "projects": {
"/active/project": {"name": "active"},
"/orphan/one": {"name": "orphan1"},
"/orphan/two": {"name": "orphan2"}
  },
  "settings": {"theme": "dark"}
}"#;
    let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    let orphaned = vec!["/orphan/one".to_string(), "/orphan/two".to_string()];

    remove_orphaned_entries(&mut parsed, &orphaned);

    // Verify in-memory mutation
    let projects = parsed["projects"].as_object().unwrap();
    assert_eq!(projects.len(), 1);
    assert!(projects.contains_key("/active/project"));
    assert!(!projects.contains_key("/orphan/one"));
    assert!(!projects.contains_key("/orphan/two"));
    // Other top-level fields preserved
    assert_eq!(parsed["version"], "1.0");
    assert_eq!(parsed["settings"]["theme"], "dark");
}

#[test]
fn test_remove_entries_no_projects_key() {
    let content = r#"{"version": "1.0"}"#;
    let mut parsed: serde_json::Value = serde_json::from_str(content).unwrap();
    let orphaned = vec!["/orphan/one".to_string()];

    // Should succeed without error (no-op)
    remove_orphaned_entries(&mut parsed, &orphaned);
}

#[test]
fn test_remove_entries_no_disk_write() {
    // Verify that remove_orphaned_entries is purely in-memory:
    // mutate the JSON, then check the in-memory value (no file path needed).
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/keep/this": {"name": "keeper"},
            "/remove/this": {"name": "goner"}
        }
    });
    let orphaned = vec!["/remove/this".to_string()];

    remove_orphaned_entries(&mut parsed, &orphaned);

    let projects = parsed["projects"].as_object().unwrap();
    assert_eq!(projects.len(), 1);
    assert!(projects.contains_key("/keep/this"));
    assert!(!projects.contains_key("/remove/this"));
}

// ── clear_stale_history tests ─────────────────────────────────

#[test]
fn test_clear_stale_history_basic() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/stale/one": {"history": ["cmd1", "cmd2", "cmd3"]},
            "/active/two": {"history": ["recent"]}
        }
    });

    let stale_paths = vec!["/stale/one".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Stale path's history should be cleared.
    let stale_history = parsed["projects"]["/stale/one"]["history"].as_array().unwrap();
    assert!(stale_history.is_empty());

    // Active path's history should be untouched.
    let active_history = parsed["projects"]["/active/two"]["history"].as_array().unwrap();
    assert_eq!(active_history.len(), 1);
}

#[test]
fn test_clear_stale_history_preserves_entry() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/stale/proj": {
                "name": "my-project",
                "history": ["old-cmd"],
                "settings": {"key": "value"}
            }
        }
    });

    let stale_paths = vec!["/stale/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Project entry still exists with other fields preserved.
    let proj = &parsed["projects"]["/stale/proj"];
    assert_eq!(proj["name"], "my-project");
    assert_eq!(proj["settings"]["key"], "value");
    // Only history is cleared.
    assert!(proj["history"].as_array().unwrap().is_empty());
}

#[test]
fn test_clear_stale_history_non_array() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/bad/proj": {"history": "not-an-array"}
        }
    });

    let stale_paths = vec!["/bad/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // history should remain unchanged (not an array, so skipped).
    assert_eq!(parsed["projects"]["/bad/proj"]["history"], "not-an-array");
}

#[test]
fn test_clear_stale_history_missing_path() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/exists/proj": {"history": ["cmd1"]}
        }
    });

    // Path not in JSON — no error.
    let stale_paths = vec!["/nonexistent/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Existing project should be untouched.
    let history = parsed["projects"]["/exists/proj"]["history"].as_array().unwrap();
    assert_eq!(history.len(), 1);
}

#[test]
fn test_clear_stale_history_no_history_key() {
    let mut parsed: serde_json::Value = serde_json::json!({
        "projects": {
            "/no-history/proj": {"name": "proj-without-history"}
        }
    });

    let stale_paths = vec!["/no-history/proj".to_string()];
    clear_stale_history(&mut parsed, &stale_paths);

    // Project should be untouched (no history key to clear).
    assert_eq!(parsed["projects"]["/no-history/proj"]["name"], "proj-without-history");
    assert!(parsed["projects"]["/no-history/proj"].get("history").is_none());
}

#[test]
fn test_clear_stale_history_no_projects_key() {
    let mut parsed: serde_json::Value = serde_json::json!({"version": "1.0"});
    let stale_paths = vec!["/some/path".to_string()];

    // Should not panic or error.
    clear_stale_history(&mut parsed, &stale_paths);

    assert_eq!(parsed["version"], "1.0");
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

// ── resolve_target_profiles tests ──────────────────────────────

#[test]
fn test_resolve_default_no_flags() {
    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: false,
    };

    let result = resolve_target_profiles(&options).unwrap();

    assert_eq!(result.len(), 1);
    let (display_name, path) = &result[0];
    // Default path should be ~/.claude
    assert!(path.ends_with(".claude"));
    // Display name should use tilde contraction
    assert!(display_name.starts_with("~"), "expected tilde path, got: {display_name}");
}

#[test]
fn test_resolve_profile_not_found() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("work".to_string(), Profile {
        name: "work".to_string(),
        config_dir: PathBuf::from("/home/user/.claude-work"),
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: Some("nonexistent".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg);

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("not found"),
        "expected 'not found' in error, got: {err_msg}"
    );
}

#[test]
fn test_resolve_profile_found() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("work".to_string(), Profile {
        name: "work".to_string(),
        config_dir: PathBuf::from("/home/user/.claude-work"),
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: Some("work".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].0, "work");
    assert_eq!(result[0].1, PathBuf::from("/home/user/.claude-work"));
}

#[test]
fn test_resolve_all_with_profiles() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("main".to_string(), Profile {
        name: "main".to_string(),
        config_dir: PathBuf::from("/home/user/.claude"),
    });
    profiles.insert("work".to_string(), Profile {
        name: "work".to_string(),
        config_dir: PathBuf::from("/home/user/.claude-work"),
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: true,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    // BTreeMap is sorted by key, so "main" comes before "work"
    assert_eq!(result.len(), 2);
    assert_eq!(result[0].0, "main");
    assert_eq!(result[1].0, "work");
}

#[test]
fn test_resolve_all_no_profiles() {
    use std::collections::BTreeMap;
    let cfg = CyoloConfig { default: None, profiles: BTreeMap::new() };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: None,
        all: true,
    };

    let result = resolve_profiles_from_config(&options, cfg);
    assert!(result.is_err());
}

#[test]
fn test_resolve_profile_tilde_expansion() {
    use crate::config::Profile;
    use std::collections::BTreeMap;

    let mut profiles = BTreeMap::new();
    profiles.insert("custom".to_string(), Profile {
        name: "custom".to_string(),
        config_dir: PathBuf::from("~/.claude-custom"),
    });
    let cfg = CyoloConfig { default: None, profiles };

    let options = DietOptions {
        apply: false,
        force: false,
        stale_days: None,
        cache: false,
        profile: Some("custom".to_string()),
        all: false,
    };

    let result = resolve_profiles_from_config(&options, cfg).unwrap();

    assert_eq!(result.len(), 1);
    // Should NOT start with ~ (tilde should be expanded)
    assert!(
        !result[0].1.to_string_lossy().starts_with('~'),
        "expected expanded path, got: {}",
        result[0].1.display()
    );
}

// ── stale/cache report tests ──────────────────────────────

/// Helper: create a DietReport with stale projects and/or cache dirs.
fn make_report_full(
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

#[test]
fn test_report_with_stale_projects() {
    setup();
    let stale = vec![
        StaleProject {
            path: "/fakehome/work/old-client".to_string(),
            last_activity_secs: 120 * 86400, // 120 days
            history_size: 340_000,
            session_size: 10_000,
        },
        StaleProject {
            path: "/fakehome/tmp/experiment".to_string(),
            last_activity_secs: 91 * 86400, // 91 days
            history_size: 580_000,
            session_size: 5_000,
        },
    ];
    let report = make_report_full(&[], vec![], stale, vec![], 3);
    let output = build_report_string(&report, false);

    assert!(output.contains("stale projects (2):"));
    assert!(output.contains("120 days ago"));
    assert!(output.contains("91 days ago"));
    assert!(output.contains("/fakehome/work/old-client"));
    assert!(output.contains("/fakehome/tmp/experiment"));
    assert!(output.contains("(history clearable)"));
}

#[test]
fn test_report_with_cache_dirs() {
    setup();
    let caches = vec![
        CacheDir {
            name: "statsig".to_string(),
            path: PathBuf::from("/fakehome/.claude/statsig"),
            size: 12_582_912, // ~12 MB
        },
        CacheDir {
            name: "file-history".to_string(),
            path: PathBuf::from("/fakehome/.claude/file-history"),
            size: 33_554_432, // ~32 MB
        },
    ];
    let report = make_report_full(&[], vec![], vec![], caches, 3);
    let output = build_report_string(&report, false);

    assert!(output.contains("clearable cache dirs (2):"));
    assert!(output.contains("statsig/"));
    assert!(output.contains("file-history/"));
    assert!(output.contains("(removable)"));
}

#[test]
fn test_report_stale_and_cache() {
    setup();
    let stale = vec![StaleProject {
        path: "/fakehome/old".to_string(),
        last_activity_secs: 100 * 86400,
        history_size: 1000,
        session_size: 500,
    }];
    let caches = vec![CacheDir {
        name: "statsig".to_string(),
        path: PathBuf::from("/fakehome/.claude/statsig"),
        size: 2000,
    }];
    let report = make_report_full(&[], vec![], stale, caches, 1);
    let output = build_report_string(&report, false);

    // Both sections present
    assert!(output.contains("stale projects (1):"));
    assert!(output.contains("clearable cache dirs (1):"));
}

#[test]
fn test_report_total_reclaimable_all() {
    setup();
    let stale = vec![StaleProject {
        path: "/fakehome/stale".to_string(),
        last_activity_secs: 200 * 86400,
        history_size: 1000,
        session_size: 2000,
    }];
    let caches = vec![CacheDir {
        name: "statsig".to_string(),
        path: PathBuf::from("/fakehome/.claude/statsig"),
        size: 3000,
    }];
    let sessions = vec![OrphanedSession {
        folder_path: PathBuf::from("/fakehome/.claude/projects/-x"),
        total_size: 4000,
    }];
    // orphan_entry_size=500, session=4000, stale_history=1000, stale_session=2000, cache=3000
    // total = 500 + 4000 + 1000 + 2000 + 3000 = 10500
    let report = make_report_full(
        &[("/fakehome/x", 500)],
        sessions,
        stale,
        caches,
        1,
    );
    let output = build_report_string(&report, false);

    assert!(output.contains("Total reclaimable:"));
    assert!(output.contains("10.3 KB"));
}

#[test]
fn test_report_nothing_to_clean_all_empty() {
    setup();
    let report = make_report_full(&[], vec![], vec![], vec![], 5);
    let output = build_report_string(&report, false);

    assert!(output.contains("No orphaned projects found. Nothing to clean up."));
    assert!(!output.contains("├─"));
    assert!(!output.contains("Total reclaimable"));
}

#[test]
fn test_report_days_ago_format() {
    setup();
    let stale = vec![StaleProject {
        path: "/fakehome/old-proj".to_string(),
        last_activity_secs: 45 * 86400, // 45 days
        history_size: 100,
        session_size: 200,
    }];
    let report = make_report_full(&[], vec![], stale, vec![], 1);
    let output = build_report_string(&report, false);

    assert!(output.contains("45 days ago"));
}
