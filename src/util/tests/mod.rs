//! Test aggregator — one `#[test]` per sibling file, declared here so
//! Rust discovers them.

mod test_expand_tilde_bare;
mod test_expand_tilde_with_suffix;
mod test_expand_tilde_absolute;
mod test_expand_tilde_relative;
