use super::super::*;
use super::common::*;

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
