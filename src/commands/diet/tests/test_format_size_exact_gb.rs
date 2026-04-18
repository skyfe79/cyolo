use super::super::*;
use super::common::*;

#[test]
fn test_format_size_exact_gb() {
    assert_eq!(format_size(1_073_741_824), "1.0 GB");
}
