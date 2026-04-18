use super::super::*;
use super::common::*;

#[test]
fn test_format_size_just_below_gb() {
    // 1024^3 - 1 = 1_073_741_823 should promote to GB, not show "1024.0 MB"
    assert_eq!(format_size(1_073_741_823), "1.0 GB");
}
