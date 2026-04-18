use super::super::*;
use super::common::*;

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
