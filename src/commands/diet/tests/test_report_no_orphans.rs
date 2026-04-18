use super::super::*;
use super::common::*;

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
