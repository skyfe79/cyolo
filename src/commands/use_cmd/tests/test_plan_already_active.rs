use super::super::*;

/// When the requested version is already the active build, the plan is a no-op
/// — even if it also appears in the installed list.
#[test]
fn test_plan_already_active() {
    let installed = vec!["2.1.157".to_string(), "2.1.158".to_string()];
    assert_eq!(
        plan("2.1.158", &installed, Some("2.1.158")),
        Plan::AlreadyActive
    );
}
