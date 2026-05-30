//! Subcommand implementations for cyolo's cyolo-owned verbs:
//! `profile` (multi-account management), `diet` (config cleanup), and the
//! native-install version verbs `version` (list) + `update` (switch).
//!
//! Each subcommand tree uses `clap` for parsing — this is where the
//! `--help` at every level comes from. The top-level router in
//! [`crate::cli`] keeps its manual classify to preserve pass-through of
//! unknown args to `claude`.

pub mod diet;
pub mod native;
pub mod profile;
pub mod update;
pub mod version;
