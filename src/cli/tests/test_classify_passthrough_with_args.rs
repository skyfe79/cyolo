use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_classify_passthrough_with_args() {
    assert_eq!(
        classify(&args(&["-p", "hello world"])),
        Command::Claude(args(&["-p", "hello world"]))
    );
}
