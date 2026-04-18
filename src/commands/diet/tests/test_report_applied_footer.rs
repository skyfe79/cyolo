use super::super::*;
use super::common::*;

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
