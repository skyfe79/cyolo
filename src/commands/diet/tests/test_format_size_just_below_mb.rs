use super::super::*;
use super::common::*;

#[test]
fn test_format_size_just_below_mb() {
    // 1024*1024 - 1 = 1_048_575 should promote to MB, not show "1024.0 KB"
    assert_eq!(format_size(1_048_575), "1.0 MB");
}
