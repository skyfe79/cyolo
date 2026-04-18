//! `cyolo profile list` — tabulate registered profiles, flag the default,
//! and show each one's bound OAuth email (or `(needs login)`).
//!
//! Also owns the shared [`read_oauth_email`] helper because `list` is its
//! primary caller; `whoami` and `picker` reach in via `super::list::*`.

use std::path::Path;

use owo_colors::OwoColorize;
use serde_json::Value;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;

/// Read `oauthAccount.emailAddress` from `<config_dir>/.claude.json` and
/// return it when present. Silently returns `None` when the file is
/// missing, unreadable, or does not contain the expected nested field —
/// this is a best-effort status read.
pub fn read_oauth_email(config_dir: &Path) -> Option<String> {
    let path = config_dir.join(".claude.json");
    let bytes = std::fs::read(&path).ok()?;
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    value
        .get("oauthAccount")?
        .get("emailAddress")?
        .as_str()
        .map(str::to_owned)
}

pub fn run() -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    if cfg.profiles.is_empty() {
        println!(
            "No profiles registered. {}",
            "Run: cyolo profile add <name>".dimmed()
        );
        return Ok(());
    }

    let max_width = cfg.profiles.keys().map(|k| k.len()).max().unwrap_or(0);

    for (name, profile) in &cfg.profiles {
        let padded = format!("{name:<max_width$}");
        let dir = profile.config_dir.display();
        let status = match read_oauth_email(&profile.config_dir) {
            Some(email) => format!("{}", email.green()),
            None => format!("{}", "(needs login)".yellow()),
        };
        if cfg.default.as_deref() == Some(name.as_str()) {
            println!(
                "{} {} -> {}  {}",
                "*".green().bold(),
                padded.bold(),
                dir,
                status
            );
        } else {
            println!("  {} -> {}  {}", padded.bold(), dir, status);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
