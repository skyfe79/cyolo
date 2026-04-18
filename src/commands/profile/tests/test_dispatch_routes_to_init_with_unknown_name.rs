use super::super::*;
use crate::error::CyoloError;

fn to_owned(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `init <unregistered-name>` must surface `ProfileNotFound` so the user
/// gets the "Run: cyolo profile add <name>" hint, not a clap
/// unknown-subcommand error.
#[test]
fn test_dispatch_routes_to_init_with_unknown_name() {
    owo_colors::set_override(false);
    let err = dispatch(&to_owned(&["init", "__test_no_such_profile__"])).unwrap_err();
    match err {
        CyoloError::ProfileNotFound { name } => {
            assert_eq!(name, "__test_no_such_profile__");
        }
        other => panic!("expected ProfileNotFound, got {other:?}"),
    }
}
