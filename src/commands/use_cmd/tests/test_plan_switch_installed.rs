use super::super::*;

/// Installed but not active → repoint, no download.
#[test]
fn test_plan_switch_installed() {
    let installed = vec!["2.1.157".to_string(), "2.1.158".to_string()];
    assert_eq!(
        plan("2.1.157", &installed, Some("2.1.158")),
        Plan::SwitchInstalled
    );
    // Also a switch when nothing is currently active.
    assert_eq!(
        plan("2.1.157", &installed, None),
        Plan::SwitchInstalled
    );
}
