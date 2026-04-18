//! `cyolo profile sync-mcp <name|--all>` — copy the `mcpServers` object
//! from `~/.claude.json` into one (or every) profile's `.claude.json`.
//!
//! Rationale lives in `crate::mcp` — TL;DR: Claude Code reads
//! `mcpServers` from `<CLAUDE_CONFIG_DIR>/.claude.json` (not the env-unset
//! `~/.claude.json`), so a fresh profile's MCP list shows up empty unless
//! cyolo seeds it.

use std::path::{Path, PathBuf};

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Registered profile name to sync. Conflicts with `--all`.
    #[arg(conflicts_with = "all")]
    pub name: Option<String>,
    /// Sync every registered profile plus `~/.claude` itself.
    #[arg(long)]
    pub all: bool,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    if !args.all && args.name.is_none() {
        eprintln!(
            "{} cyolo profile sync-mcp <name|--all>",
            "Usage:".yellow().bold()
        );
        return Err(CyoloError::NonZeroExit(1));
    }

    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    if args.all {
        // Every registered profile, plus `~/.claude` itself so the picker's
        // `d) default` path reads the same User MCPs the user already has
        // in `~/.claude.json`.
        let mut targets: Vec<(String, PathBuf)> = cfg
            .profiles
            .iter()
            .map(|(name, p)| (name.clone(), p.config_dir.clone()))
            .collect();
        if let Some(home) = dirs::home_dir() {
            let source_dir = home.join(".claude");
            if !targets.iter().any(|(_, p)| p == &source_dir) {
                targets.push(("(default ~/.claude)".to_string(), source_dir));
            }
        }
        if targets.is_empty() {
            println!(
                "{}",
                "No profiles registered. Run: cyolo profile add <name>".dimmed()
            );
            return Ok(());
        }
        for (name, dir) in &targets {
            sync_one_profile(name, dir);
        }
        Ok(())
    } else {
        let name = args.name.expect("checked above");
        let profile = cfg
            .profiles
            .get(&name)
            .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;
        let config_dir = crate::util::expand_tilde(&profile.config_dir.to_string_lossy());
        sync_one_profile(&name, &config_dir);
        Ok(())
    }
}

/// Run the MCP sync for a single profile and print a one-line report.
///
/// Failures stay local (warning + keep going) so that a malformed
/// `.claude.json` on one profile does not halt a `--all` sweep.
fn sync_one_profile(name: &str, config_dir: &Path) {
    match crate::mcp::sync_mcp_to_profile(config_dir) {
        Ok(0) => println!(
            "  {} {}  {}",
            name.bold(),
            "→".dimmed(),
            "nothing to sync (source missing or empty)".dimmed()
        ),
        Ok(n) => println!(
            "  {} {}  synced {} MCP server(s) into {}",
            name.green().bold(),
            "→".dimmed(),
            n,
            format!("{}/.claude.json", config_dir.display()).dimmed(),
        ),
        Err(e) => eprintln!(
            "  {} {}  {} {}",
            name.red().bold(),
            "→".dimmed(),
            "sync failed:".yellow().bold(),
            e
        ),
    }
}

/// Best-effort wrapper used by `profile::add::run` and the picker's
/// `d) default` branch — calls `mcp::sync_mcp_to_profile` and prints a
/// compact status line for non-zero syncs, swallows errors into a warning.
pub fn report_mcp_sync(config_dir: &Path) {
    match crate::mcp::sync_mcp_to_profile(config_dir) {
        Ok(0) => {}
        Ok(n) => println!(
            "{} synced {} User MCP server(s) from {} into {}",
            "↳".dimmed(),
            n,
            "~/.claude.json".dimmed(),
            format!("{}/.claude.json", config_dir.display()).dimmed(),
        ),
        Err(e) => eprintln!(
            "{} could not sync user MCPs: {}",
            "warning:".yellow().bold(),
            e
        ),
    }
}

#[cfg(test)]
mod tests;
