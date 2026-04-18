use super::super::*;
use super::common::*;

#[test]
fn test_is_claude_running_returns_bool() {
    // Verify the function compiles and returns a bool without panicking.
    // Cannot deterministically test true/false since pgrep depends on runtime state.
    let _result: bool = is_claude_running();
}
