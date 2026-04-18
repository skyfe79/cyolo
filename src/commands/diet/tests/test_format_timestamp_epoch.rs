use super::super::*;
use super::common::*;

#[test]
fn test_format_timestamp_epoch() {
    // Unix epoch: 1970-01-01 00:00:00
    assert_eq!(format_timestamp(0), "19700101000000");
}
