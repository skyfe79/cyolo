//! The interactive profile picker fired by
//!
//!   * bare `cyolo` (via `cli::route`) when nothing resolved and the
//!     terminal is a TTY;
//!   * `cyolo profile init` with no name and no default set.
//!
//! The picker is deliberately the single place where menu shape, input
//! parsing, and the "new profile + login" escape hatch live — that way
//! the two callers above stay thin.

use owo_colors::OwoColorize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;
use crate::util::read_line_trimmed;

use super::list::read_oauth_email;
use super::marker::{write_default_marker, write_profile_marker};
use super::sync_mcp::report_mcp_sync;

/// What the interactive init menu resolved from the user's input line.
#[derive(Debug, PartialEq)]
pub enum MenuChoice {
    /// Zero-based index into the sorted profile list.
    Pick(usize),
    /// Register a fresh profile then bind to it.
    New,
    /// Pin this directory to Claude Code's own default config (`~/.claude`)
    /// by writing an inline-`config_dir` marker. Always available even when
    /// no profiles are registered.
    Default,
    /// Do nothing, exit cleanly.
    Quit,
    /// Input did not match any option.
    Invalid,
}

/// Outcome of [`interactive_init_menu`] — distinguishes the "already launched
/// claude during the picker" case from the "just wrote a marker" case so a
/// caller that was about to run claude itself (bare `cyolo`) can avoid a
/// surprise double-launch.
#[derive(Debug, PartialEq)]
pub enum PickerOutcome {
    /// User picked an existing registered profile; `.claude-profile.json` was
    /// written. Caller should re-resolve and launch claude normally.
    MarkerWritten,
    /// User chose "new": a fresh profile was registered and `claude /login`
    /// was already launched (and exited) inside `add::run()`, and then the
    /// marker was written. Caller should **not** launch claude again.
    NewProfileRegistered,
    /// User quit without doing anything. No marker written.
    Quit,
}

/// Parse one line of user input from the interactive init menu.
///
/// Accepts:
///   * `<digit>`         — 1-based index; returned as 0-based `Pick`
///   * `n` / `new`       — `New`
///   * `d` / `default`   — `Default` (pin to `~/.claude`)
///   * `q` / `quit`      — `Quit`
///   * empty line        — `Quit` (treat blank enter as "not now")
///   * anything else     — `Invalid`
pub fn parse_menu_input(input: &str, profile_count: usize) -> MenuChoice {
    let s = input.trim().to_lowercase();
    if s.is_empty() || s == "q" || s == "quit" {
        return MenuChoice::Quit;
    }
    if s == "n" || s == "new" {
        return MenuChoice::New;
    }
    if s == "d" || s == "default" {
        return MenuChoice::Default;
    }
    if let Ok(n) = s.parse::<usize>()
        && n >= 1
        && n <= profile_count
    {
        return MenuChoice::Pick(n - 1);
    }
    MenuChoice::Invalid
}

/// Show the interactive profile picker and apply the user's choice.
///
/// Caller must have confirmed we are on a TTY.  Returns a [`PickerOutcome`]
/// describing what was done so the caller can avoid double-launching
/// claude (see `PickerOutcome::NewProfileRegistered`).
pub fn interactive_init_menu() -> Result<PickerOutcome, CyoloError> {
    let cfg = CyoloConfig::load()?;

    // Sorted profile names for stable indexing across invocations.
    let names: Vec<String> = cfg.profiles.keys().cloned().collect();
    let width = names.iter().map(String::len).max().unwrap_or(0);

    println!(
        "{} no profile is bound to this directory. Pick one:",
        "ℹ".cyan().bold()
    );
    println!();

    if names.is_empty() {
        println!("  {}", "(no profiles registered yet)".dimmed());
    } else {
        for (i, name) in names.iter().enumerate() {
            let profile = &cfg.profiles[name];
            let email = read_oauth_email(&profile.config_dir)
                .map(|e| e.green().to_string())
                .unwrap_or_else(|| "(needs login)".yellow().to_string());
            println!(
                "  {index}) {pad}  {email}",
                index = (i + 1).to_string().bold(),
                pad = format!("{name:<width$}").bold(),
            );
        }
    }
    println!("  {}) {}", "n".bold(), "new      register a new profile + /login");
    println!("  {}) {}", "d".bold(), "default  pin this directory to ~/.claude (Claude Code default)");
    println!("  {}) {}", "q".bold(), "quit     do nothing");
    println!();

    use std::io::Write as _;
    print!("{} ", "Selection:".bold());
    std::io::stdout().flush().ok();

    let raw = read_line_trimmed()?;
    match parse_menu_input(&raw, names.len()) {
        MenuChoice::Pick(i) => {
            write_profile_marker(&names[i])?;
            Ok(PickerOutcome::MarkerWritten)
        }
        MenuChoice::New => {
            print!("{} ", "Name for new profile:".bold());
            std::io::stdout().flush().ok();
            let new_name = read_line_trimmed()?;
            if new_name.is_empty() {
                eprintln!("{} profile name cannot be empty", "error:".red().bold());
                return Err(CyoloError::NonZeroExit(1));
            }
            // `add::run` registers the profile *and* launches claude for
            // `/login`. That login session is a blocking, interactive run of
            // claude, so the caller of this picker must not double-launch.
            super::add::run(super::add::Args {
                name: new_name.clone(),
                config_dir: None,
                no_share: false,
                no_login: false,
            })?;
            write_profile_marker(&new_name)?;
            Ok(PickerOutcome::NewProfileRegistered)
        }
        MenuChoice::Default => {
            write_default_marker()?;
            // Seed `~/.claude/.claude.json` — picking `d` sets
            // `CLAUDE_CONFIG_DIR=~/.claude` at runtime, which makes claude
            // read that file instead of `$HOME/.claude.json`, so the User
            // MCPs need to be mirrored into it.
            if let Some(home) = dirs::home_dir() {
                report_mcp_sync(&home.join(".claude"));
            }
            Ok(PickerOutcome::MarkerWritten)
        }
        MenuChoice::Quit => {
            println!(
                "{}",
                "No change. Run `cyolo profile init <name>` when ready.".dimmed()
            );
            Ok(PickerOutcome::Quit)
        }
        MenuChoice::Invalid => {
            eprintln!(
                "{} unrecognized selection '{}'",
                "error:".red().bold(),
                raw.bold()
            );
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

#[cfg(test)]
mod tests;
