use super::super::*;
use super::common::*;

#[test]
fn test_format_size_gb() {
    assert_eq!(format_size(1_500_000_000), "1.4 GB");
}
