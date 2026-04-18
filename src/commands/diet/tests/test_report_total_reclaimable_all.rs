use super::super::*;
use super::common::*;

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
