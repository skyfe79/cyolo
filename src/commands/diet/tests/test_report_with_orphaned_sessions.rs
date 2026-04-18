use super::super::*;
use super::common::*;

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
