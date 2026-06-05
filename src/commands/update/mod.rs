//! `cyolo update` — update to the latest Claude Code build.
//!
//! Bare `cyolo update` (and any flag form) is NOT routed here: the top-level
//! classifier forwards it straight to `claude update`, Claude Code's own
//! auto-updater, which fetches the newest build. This module exists only to
//! catch the *legacy* `cyolo update <version>` form — version switching moved
//! to the dedicated [`crate::commands::use_cmd`] verb (`cyolo use <version>`),
//! which also downloads a missing build. We intercept the old positional form
//! and redirect the user there rather than passing a version `claude update`
//! doesn't understand.

use owo_colors::OwoColorize;

use crate::error::CyoloError;

/// Route `cyolo update <version>` (legacy switch form) → redirect to `cyolo use`.
///
/// The classifier only sends us a positional `<version>`; bare `update` and
/// flag forms go to `claude update`. We still guard the empty case defensively.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let version = args.first().map(|s| s.as_str()).unwrap_or("<version>");

    eprintln!(
        "{} `cyolo update <version>` no longer switches versions.",
        "note:".yellow().bold()
    );
    eprintln!(
        "  Switch to a specific build with {} (it downloads the build if missing).",
        format!("`cyolo use {version}`").bold()
    );
    eprintln!(
        "  Bare {} updates to the latest build (Claude Code's native updater).",
        "`cyolo update`".bold()
    );
    Err(CyoloError::NonZeroExit(2))
}

#[cfg(test)]
mod tests;
