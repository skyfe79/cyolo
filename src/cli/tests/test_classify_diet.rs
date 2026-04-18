use super::super::*;

fn args(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn test_classify_diet() {
    assert_eq!(
        classify(&args(&["diet", "--apply"])),
        Command::Diet(args(&["--apply"]))
    );
}
