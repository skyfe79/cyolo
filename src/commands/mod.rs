//! Subcommand implementations for cyolo's two cyolo-owned verbs:
//! `profile` (multi-account management) and `diet` (config cleanup).
//!
//! Each subcommand tree uses `clap` for parsing — this is where the
//! `--help` at every level comes from. The top-level router in
//! [`crate::cli`] keeps its manual classify to preserve pass-through of
//! unknown args to `claude`.

pub mod diet;
pub mod profile;
