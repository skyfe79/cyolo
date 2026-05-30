use super::super::*;

/// Numeric ordering, not lexical: 2.1.100 must outrank 2.1.9, and the list
/// comes back newest-first.
#[test]
fn test_sort_versions_newest_first() {
    let mut v = vec![
        "2.1.9".to_string(),
        "2.1.100".to_string(),
        "2.1.10".to_string(),
        "2.0.5".to_string(),
        "10.0.0".to_string(),
    ];
    sort_versions(&mut v);
    assert_eq!(
        v,
        vec![
            "10.0.0".to_string(),
            "2.1.100".to_string(),
            "2.1.10".to_string(),
            "2.1.9".to_string(),
            "2.0.5".to_string(),
        ]
    );
}
