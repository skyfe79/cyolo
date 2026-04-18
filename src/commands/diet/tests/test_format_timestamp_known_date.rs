use super::super::*;
use super::common::*;

#[test]
fn test_format_timestamp_known_date() {
    // 1700000000 = 2023-11-14 22:13:20 UTC
    assert_eq!(format_timestamp(1_700_000_000), "20231114221320");
}
