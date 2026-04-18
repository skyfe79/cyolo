use super::super::*;
use super::common::*;

#[test]
fn test_dispatch_unknown_arg_returns_error() {
    let args = vec!["--unknown".to_string()];
    let result = dispatch(&args);
    assert!(result.is_err());
}
