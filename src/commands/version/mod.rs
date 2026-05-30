//! `cyolo version [ls [remote]]` — inspect installed (and upstream) Claude
//! Code versions.
//!
//! - `cyolo version`            → show the currently-linked version
//! - `cyolo version ls`         → list locally-installed versions
//! - `cyolo version ls remote`  → list upstream releases from the npm registry,
//!   marking which are already installed
//!
//! Switching between versions lives in the sibling [`crate::commands::update`]
//! verb; this module is read-only.

use clap::{Parser, Subcommand, ValueEnum};
use owo_colors::OwoColorize;

use crate::commands::native;
use crate::error::CyoloError;

/// Root clap struct for `cyolo version <...>`. Mirrors the `profile`/`diet`
/// trees: `no_binary_name` so clap reads our verbatim args, and the in-house
/// version flag is disabled because `version` IS the verb here.
#[derive(Parser, Debug)]
#[command(
    name = "version",
    about = "List installed (and upstream) Claude Code versions",
    no_binary_name = true,
    disable_version_flag = true
)]
pub struct VersionCli {
    #[command(subcommand)]
    pub command: Option<VersionCommand>,
}

#[derive(Subcommand, Debug)]
pub enum VersionCommand {
    /// List installed versions; add `remote` to list upstream releases.
    #[command(alias = "ls")]
    List {
        /// Where to list from: `local` (default) or `remote`.
        #[arg(value_enum, default_value_t = Source::Local)]
        source: Source,
    },
}

#[derive(ValueEnum, Clone, Debug, PartialEq)]
pub enum Source {
    /// Versions present under the local `versions/` directory.
    Local,
    /// Versions published upstream (npm registry).
    Remote,
}

/// Route `cyolo version <...>`.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let cli = match VersionCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => return handle_clap_error(e),
    };

    match cli.command {
        None => show_current(),
        Some(VersionCommand::List {
            source: Source::Local,
        }) => list_local(),
        Some(VersionCommand::List {
            source: Source::Remote,
        }) => list_remote(),
    }
}

/// Bare `cyolo version` → print the active version + a pointer to `ls`.
fn show_current() -> Result<(), CyoloError> {
    let install = native::discover()?;
    match &install.current {
        Some(v) => println!("{} {}", "current:".bold(), v.green().bold()),
        None => println!("current version could not be determined"),
    }
    println!(
        "{}",
        "Run `cyolo version ls` to list all installed versions.".dimmed()
    );
    Ok(())
}

/// `cyolo version ls` → list local versions, marking the current one.
fn list_local() -> Result<(), CyoloError> {
    let install = native::discover()?;
    let versions = native::installed_versions_in(&install.versions_dir)?;
    if versions.is_empty() {
        println!(
            "No installed versions found under {}",
            install.versions_dir.display()
        );
        return Ok(());
    }
    println!("{}", "Installed versions:".bold());
    for v in &versions {
        if install.current.as_deref() == Some(v.as_str()) {
            println!(
                "  {} {}  {}",
                "●".green(),
                v.green().bold(),
                "(current)".dimmed()
            );
        } else {
            println!("  {} {}", "○".dimmed(), v);
        }
    }
    println!();
    println!("{}", "Switch with `cyolo update <version>`.".dimmed());
    Ok(())
}

/// `cyolo version ls remote` → list upstream releases, flagging which are
/// already installed locally / current / latest.
fn list_remote() -> Result<(), CyoloError> {
    // Local context is best-effort: if there's no native install we can still
    // show the upstream list, just without installed/current annotations.
    let install = native::discover().ok();
    let installed = install
        .as_ref()
        .and_then(|i| native::installed_versions_in(&i.versions_dir).ok())
        .unwrap_or_default();
    let current = install.as_ref().and_then(|i| i.current.clone());

    let (versions, latest) = fetch_remote_versions()?;
    let total = versions.len();
    const LIMIT: usize = 20;

    println!("{}", "Upstream versions (npm registry):".bold());
    for v in versions.iter().take(LIMIT) {
        let mut tags: Vec<String> = Vec::new();
        if current.as_deref() == Some(v.as_str()) {
            tags.push("current".green().to_string());
        } else if installed.contains(v) {
            tags.push("installed".cyan().to_string());
        }
        if latest.as_deref() == Some(v.as_str()) {
            tags.push("latest".yellow().to_string());
        }
        let suffix = if tags.is_empty() {
            String::new()
        } else {
            format!("  ({})", tags.join(", "))
        };
        let bullet = if installed.contains(v) {
            "●".green().to_string()
        } else {
            "○".dimmed().to_string()
        };
        println!("  {bullet} {v}{suffix}");
    }
    if total > LIMIT {
        println!(
            "{}",
            format!("  … and {} older releases", total - LIMIT).dimmed()
        );
    }
    println!();
    println!(
        "{}",
        "Already-installed versions switch instantly with `cyolo update <version>`.".dimmed()
    );
    println!(
        "{}",
        "To fetch a newer build, run `cyolo update` (Claude Code's native updater).".dimmed()
    );
    Ok(())
}

/// Fetch the npm registry doc for `@anthropic-ai/claude-code` and extract its
/// version list. Shells out to `curl` (matching `diet`'s process style) rather
/// than pulling in an HTTP-client dependency; the slim
/// `application/vnd.npm.install-v1+json` doc still carries `.versions` +
/// `.dist-tags.latest` while being far smaller than the full document.
fn fetch_remote_versions() -> Result<(Vec<String>, Option<String>), CyoloError> {
    const URL: &str = "https://registry.npmjs.org/@anthropic-ai/claude-code";
    let output = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "-H",
            "Accept: application/vnd.npm.install-v1+json",
            URL,
        ])
        .output()
        .map_err(|e| CyoloError::RemoteFetchFailed {
            message: format!("could not run curl: {e}"),
        })?;
    if !output.status.success() {
        return Err(CyoloError::RemoteFetchFailed {
            message: format!(
                "curl exited with {} (no network, or registry unreachable)",
                output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "signal".to_string())
            ),
        });
    }
    let body = String::from_utf8_lossy(&output.stdout);
    parse_npm_versions(&body)
}

/// Parse an npm registry document into `(versions newest-first, latest tag)`.
/// Pure so it can be unit-tested without a network round-trip.
pub fn parse_npm_versions(body: &str) -> Result<(Vec<String>, Option<String>), CyoloError> {
    let json: serde_json::Value =
        serde_json::from_str(body).map_err(|e| CyoloError::RemoteFetchFailed {
            message: format!("invalid registry JSON: {e}"),
        })?;
    let mut versions: Vec<String> = match json.get("versions").and_then(|v| v.as_object()) {
        Some(map) => map.keys().cloned().collect(),
        None => {
            return Err(CyoloError::RemoteFetchFailed {
                message: "registry response had no `versions` object".to_string(),
            });
        }
    };
    native::sort_versions(&mut versions);
    let latest = json
        .get("dist-tags")
        .and_then(|t| t.get("latest"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok((versions, latest))
}

/// Turn a `clap::Error` into either `Ok(())` (help/version display) or
/// `Err(NonZeroExit)` (genuine parse failure) — same shape as `profile`.
fn handle_clap_error(e: clap::Error) -> Result<(), CyoloError> {
    use clap::error::ErrorKind;
    match e.kind() {
        ErrorKind::DisplayHelp
        | ErrorKind::DisplayVersion
        | ErrorKind::DisplayHelpOnMissingArgumentOrSubcommand => {
            e.print().ok();
            Ok(())
        }
        _ => {
            e.print().ok();
            Err(CyoloError::NonZeroExit(e.exit_code()))
        }
    }
}

#[cfg(test)]
mod tests;
