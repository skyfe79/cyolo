use super::super::*;
use crate::error::CyoloError;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `sync-mcp <unregistered-name>` also surfaces `ProfileNotFound`. Proves
/// clap-level routing for kebab-case names (`sync-mcp` != `sync_mcp`).
#[test]
fn test_dispatch_routes_to_sync_mcp_with_unknown_name() {
    owo_colors::set_override(false);
    let err = dispatch(&to_owned(&["sync-mcp", "__test_no_such_profile__"])).unwrap_err();
    match err {
        CyoloError::ProfileNotFound { name } => {
            assert_eq!(name, "__test_no_such_profile__");
        }
        other => panic!("expected ProfileNotFound, got {other:?}"),
    }
}
