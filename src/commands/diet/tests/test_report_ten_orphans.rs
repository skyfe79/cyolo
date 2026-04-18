use super::super::*;
use super::common::*;

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
