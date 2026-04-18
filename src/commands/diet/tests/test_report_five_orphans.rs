use super::super::*;
use super::common::*;

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
