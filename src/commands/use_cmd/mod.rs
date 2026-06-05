//! `cyolo use <version>` — make `<version>` the active Claude Code build,
//! downloading it first if it isn't installed yet.
//!
//! This is the version-*switch* verb. Unlike a plain symlink repoint, `use`
//! will *fetch* a missing version by delegating to Claude Code's own installer
//! (`claude install <version>` — see [`crate::runner::run_claude_install`]),
//! which handles platform/Rosetta/musl detection and checksum verification.
//! Once the build is on disk we repoint the launcher symlink at it via
//! [`crate::commands::native::switch_in`].
//!
//! ```text
//!   cyolo use 2.1.158         # switch to 2.1.158, downloading it if missing
//!   cyolo use 2.1.158 --yes   # don't prompt before a download
//! ```
//!
//! Switching between already-installed builds is instant (atomic symlink
//! repoint, no network). For "just give me the newest build" use
//! `cyolo update`, which passes through to claude's native auto-updater.

use owo_colors::OwoColorize;

use crate::commands::native::{self, Install};
use crate::error::CyoloError;
use crate::runner;
use crate::util;

/// What `use <version>` needs to do, decided purely from local install state so
/// the branch logic stays unit-testable without touching the filesystem or
/// network.
#[derive(Debug, PartialEq)]
enum Plan {
    /// `version` is already the active build — nothing to do.
    AlreadyActive,
    /// `version` is installed but not active — repoint the launcher symlink.
    SwitchInstalled,
    /// `version` isn't on disk — download it, then activate.
    DownloadThenActivate,
}

/// Decide the plan from the requested version and local install state. Pure.
fn plan(version: &str, installed: &[String], current: Option<&str>) -> Plan {
    if current == Some(version) {
        Plan::AlreadyActive
    } else if installed.iter().any(|v| v == version) {
        Plan::SwitchInstalled
    } else {
        Plan::DownloadThenActivate
    }
}

/// True when `s` looks like a concrete `MAJOR.MINOR.PATCH[-suffix]` version.
///
/// `use` targets a *specific* build (use `cyolo update` for "latest"), so tags
/// like `latest`/`stable` and malformed input are rejected up front rather than
/// shelling out to a download that would fail confusingly. This is a format
/// check only — it deliberately does NOT pre-validate against any registry, so
/// a brand-new upstream version is never false-rejected.
fn looks_like_version(s: &str) -> bool {
    let core = s.split('-').next().unwrap_or(s);
    let parts: Vec<&str> = core.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
}

/// Route `cyolo use <...>`.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let mut version: Option<&str> = None;
    let mut assume_yes = false;
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            "--yes" | "-y" => assume_yes = true,
            other if other.starts_with('-') => {
                eprintln!("{} unknown flag '{other}'", "error:".red().bold());
                print_help();
                return Err(CyoloError::NonZeroExit(2));
            }
            other => {
                if version.is_some() {
                    eprintln!(
                        "{} unexpected extra argument '{other}'",
                        "error:".red().bold()
                    );
                    return Err(CyoloError::NonZeroExit(2));
                }
                version = Some(other);
            }
        }
    }

    let version = match version {
        Some(v) => v,
        None => {
            eprintln!(
                "{} usage: cyolo use <version>   (see `cyolo version ls remote`)",
                "error:".red().bold()
            );
            return Err(CyoloError::NonZeroExit(2));
        }
    };

    if !looks_like_version(version) {
        eprintln!(
            "{} '{version}' is not a concrete version (expected e.g. 2.1.158).",
            "error:".red().bold()
        );
        eprintln!("  To update to the latest build, run `cyolo update`.");
        eprintln!("  Browse upstream releases with `cyolo version ls remote`.");
        return Err(CyoloError::NonZeroExit(2));
    }

    let install = native::discover()?;
    let installed = native::installed_versions_in(&install.versions_dir)?;

    match plan(version, &installed, install.current.as_deref()) {
        Plan::AlreadyActive => {
            println!("{} already on {}", "✓".green(), version.green().bold());
            Ok(())
        }
        Plan::SwitchInstalled => activate(&install, version, false),
        Plan::DownloadThenActivate => {
            if !assume_yes && util::is_interactive() && !confirm_download(version)? {
                println!("{}", "Cancelled — nothing downloaded.".dimmed());
                return Ok(());
            }
            println!(
                "{} downloading Claude Code {} …",
                "↓".cyan().bold(),
                version.bold()
            );
            runner::run_claude_install(version)?;

            // `claude install` lays the build out and usually repoints the
            // launcher itself. Re-discover so we report the real end state and
            // only repoint if it didn't already land on `version`.
            let install = native::discover()?;
            if install.current.as_deref() == Some(version) {
                println!(
                    "{} installed and now active: {}",
                    "✓".green().bold(),
                    version.green().bold()
                );
                Ok(())
            } else {
                activate(&install, version, true)
            }
        }
    }
}

/// Repoint the launcher symlink at `version` and report. `downloaded` tunes the
/// wording so a fresh fetch reads "installed and switched" vs a plain switch.
fn activate(install: &Install, version: &str, downloaded: bool) -> Result<(), CyoloError> {
    native::switch_in(&install.bin_link, &install.versions_dir, version)?;
    let verb = if downloaded {
        "installed and switched to"
    } else {
        "switched to"
    };
    println!("{} {verb} {}", "✓".green().bold(), version.green().bold());
    println!(
        "  {} → {}",
        install.bin_link.display().to_string().dimmed(),
        install
            .versions_dir
            .join(version)
            .display()
            .to_string()
            .dimmed()
    );
    println!(
        "{}",
        "Run `claude --version` in a new shell to confirm.".dimmed()
    );
    Ok(())
}

/// Ask before a multi-hundred-MB download. Defaults to yes on a bare Enter; the
/// caller only reaches here on an interactive terminal without `--yes`.
fn confirm_download(version: &str) -> Result<bool, CyoloError> {
    use std::io::Write as _;
    print!(
        "{} {version} is not installed. Download and install it now? [Y/n] ",
        "?".yellow().bold()
    );
    let _ = std::io::stdout().flush();
    let answer = util::read_line_trimmed()?.to_ascii_lowercase();
    Ok(answer.is_empty() || answer == "y" || answer == "yes")
}

/// Print cyolo's `use` help.
fn print_help() {
    println!(
        "{} — switch to a Claude Code version, downloading it if needed",
        "cyolo use".bold()
    );
    println!();
    println!("{}", "Usage:".bold());
    println!("  cyolo use <version>        Activate <version> (fetch it first if not installed)");
    println!("  cyolo use <version> --yes  Skip the download confirmation prompt");
    println!("  cyolo use --help           Show this message");
    println!();
    println!("{}", "Behavior:".bold());
    println!(
        "  {}",
        "* Already installed → instant atomic symlink repoint (no download).".dimmed()
    );
    println!(
        "  {}",
        "* Not installed yet → delegates to `claude install <version>` to fetch".dimmed()
    );
    println!(
        "  {}",
        "  and lay out the build, then activates it.".dimmed()
    );
    println!();
    println!("{}", "See also:".bold());
    println!("  cyolo version ls           List installed versions");
    println!("  cyolo version ls remote    List upstream releases (npm registry)");
    println!("  cyolo update               Update to the latest build (claude update)");
}

#[cfg(test)]
mod tests;
