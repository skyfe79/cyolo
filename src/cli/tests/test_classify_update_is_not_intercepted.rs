use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// `update` used to mean `claude update`. It was removed: now it passes
/// through like any other argument so the user can still run
/// `cyolo update` and have claude receive it verbatim.
#[test]
fn test_classify_update_is_not_intercepted() {
    assert_eq!(
        classify(&args(&["update"])),
        Command::Claude(args(&["update"]))
    );
}
