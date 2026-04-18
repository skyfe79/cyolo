use super::super::*;
use crate::config::CyoloConfig;
use std::collections::BTreeMap;

#[test]
fn test_resolve_none() {
    let config = CyoloConfig {
        default: None,
        profiles: BTreeMap::new(),
    };
    let result = resolve_with(&config, None).unwrap();
    assert!(result.is_none());
}
