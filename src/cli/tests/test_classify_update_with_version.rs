use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

/// The legacy `cyolo update <version>` positional form is still intercepted —
/// but now only to redirect the user to `cyolo use` (the redirect itself lives
/// in `update::dispatch`). The leading `update` token is stripped first.
///
/// Flag forms (`--help`/`-h`/`--check`) are NOT intercepted anymore: `update`
/// means "update to latest", so they pass through to `claude update` — see
/// `test_classify_update_is_not_intercepted`.
#[test]
fn test_classify_update_with_version() {
    assert_eq!(
        classify(&args(&["update", "2.1.156"])),
        Command::Update(args(&["2.1.156"]))
    );
}
