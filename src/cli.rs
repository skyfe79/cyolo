use owo_colors::OwoColorize;

use crate::detect;
use crate::diet;
use crate::error::CyoloError;
use crate::profile;
use crate::runner;

/// Top-level command classification.
///
/// cyolo-specific subcommands (`profile`, `diet`, `help`) are handled in
/// process. Everything else is forwarded verbatim to
/// `claude --dangerously-skip-permissions`.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// `cyolo profile <...>` → profile management
    Profile(Vec<String>),
    /// `cyolo diet <...>` → config cleanup
    Diet(Vec<String>),
    /// `cyolo help` / `cyolo --help` / `cyolo -h` → print cyolo's own help
    Help,
    /// Everything else → `claude --dangerously-skip-permissions <args>`
    Claude(Vec<String>),
}

/// Classify raw CLI arguments into a Command.
pub fn classify(args: &[String]) -> Command {
    match args.first().map(|s| s.as_str()) {
        Some("profile") => Command::Profile(args[1..].to_vec()),
        Some("diet") => Command::Diet(args[1..].to_vec()),
        Some("help") | Some("--help") | Some("-h") => Command::Help,
        _ => Command::Claude(args.to_vec()),
    }
}

/// Route execution based on CLI arguments.
pub fn route() -> Result<(), CyoloError> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match classify(&args) {
        Command::Profile(args) => profile::dispatch(&args),
        Command::Diet(args) => diet::dispatch(&args),
        Command::Help => {
            print_help();
            Ok(())
        }
        Command::Claude(args) => {
            let mut resolved = detect::resolve_profile()?;
            // Scope: only fire the picker when the user typed a bare `cyolo`
            // (no args) on an interactive terminal and nothing resolved.
            // `cyolo -p "..."` or any pass-through invocation stays silent.
            let show_picker =
                resolved.is_none() && args.is_empty() && profile::is_interactive();
            if show_picker {
                match profile::interactive_init_menu() {
                    Ok(profile::PickerOutcome::MarkerWritten) => {
                        // Re-resolve so the brand-new marker is honoured on
                        // this very same invocation — no second `cyolo` call.
                        resolved = detect::resolve_profile()?;
                    }
                    Ok(profile::PickerOutcome::NewProfileRegistered) => {
                        // `add` already ran `claude /login` interactively.
                        // Launching another claude session here would surprise
                        // the user with a back-to-back double launch — bail.
                        return Ok(());
                    }
                    Ok(profile::PickerOutcome::Quit) => {
                        // Explicit "do nothing" from the picker: exit cleanly
                        // without launching claude. The PRD §3.1 pass-through
                        // to `~/.claude` is still preserved for the non-picker
                        // paths (non-TTY stdin, or `cyolo <args...>`).
                        return Ok(());
                    }
                    Err(e) => return Err(e),
                }
            } else {
                maybe_hint_no_profile(resolved.is_none());
            }
            runner::run_claude(&args, resolved.as_ref())
        }
    }
}

/// Print cyolo's top-level help to stdout.
///
/// Covers the cyolo-owned subcommands and explicitly documents the
/// pass-through rule for everything else, so users can predict how any
/// given `cyolo <...>` invocation will be classified.
fn print_help() {
    let version = env!("CARGO_PKG_VERSION");
    println!(
        "{} {version} — Claude Code profile manager + config cleaner",
        "cyolo".bold()
    );
    println!();
    // Column widths are hand-tuned because ANSI color codes are zero-visual-
    // width but count as characters in `{:<N}` padding — so we color the name
    // AFTER composing the visible column.
    println!("{}", "Cyolo subcommands (handled in-process):".bold());
    println!("  {}{}  Manage per-account config directories", "profile".green(), " ...    ");
    println!("  {}{}  Report / reclaim orphaned project data + caches", "diet".green(), " ...       ");
    println!("  {}{}  Show this message (also: --help / -h)", "help".green(), "           ");
    println!();
    println!("{}", "Anything else is forwarded as:".bold());
    println!("  claude --dangerously-skip-permissions <args...>");
    println!();
    println!("{}", "Examples:".bold());
    println!("  cyolo                          # interactive claude with resolved profile");
    println!("  cyolo -p \"hi there\"            # one-shot prompt via claude -p");
    println!("  cyolo profile                  # detailed profile subcommand help");
    println!("  cyolo diet                     # dry-run cleanup report");
    println!();
    println!("{}", "Notes:".dimmed());
    println!("  {}", "* Run `claude update` directly — cyolo does not manage Claude Code's own version.".dimmed());
    println!("  {}", "* `cyolo --version` currently prints claude's version (pass-through).".dimmed());
    println!("  {}", "* Full docs: https://github.com/skyfe79/cyolo".dimmed());
}

/// Print a one-line heads-up to stderr when `cyolo` runs without any
/// resolvable profile (no walk-up hit, no default), but only on an
/// interactive terminal.  Scripts and pipes see nothing.
fn maybe_hint_no_profile(no_profile: bool) {
    use std::io::IsTerminal as _;
    if !no_profile {
        return;
    }
    if !std::io::stderr().is_terminal() {
        return;
    }
    eprintln!(
        "{} no profile detected — run {} to bind this directory",
        "ℹ".cyan().bold(),
        "`cyolo profile init`".bold()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_classify_profile() {
        assert_eq!(
            classify(&args(&["profile", "list"])),
            Command::Profile(args(&["list"]))
        );
    }

    #[test]
    fn test_classify_diet() {
        assert_eq!(
            classify(&args(&["diet", "--apply"])),
            Command::Diet(args(&["--apply"]))
        );
    }

    #[test]
    fn test_classify_help_variants() {
        assert_eq!(classify(&args(&["help"])), Command::Help);
        assert_eq!(classify(&args(&["--help"])), Command::Help);
        assert_eq!(classify(&args(&["-h"])), Command::Help);
    }

    #[test]
    fn test_classify_passthrough_with_args() {
        assert_eq!(
            classify(&args(&["-p", "hello world"])),
            Command::Claude(args(&["-p", "hello world"]))
        );
    }

    #[test]
    fn test_classify_no_args() {
        // Bare `cyolo` launches the interactive claude session; no help or update
        // short-circuit should intervene.
        assert_eq!(classify(&args(&[])), Command::Claude(vec![]));
    }

    #[test]
    fn test_classify_update_is_not_intercepted() {
        // `update` used to mean `claude update`. It was removed: now it
        // passes through like any other argument so the user can still run
        // `cyolo update` and have claude receive it (claude itself will
        // handle or reject the verb). Importantly, cyolo no longer special-
        // cases it.
        assert_eq!(
            classify(&args(&["update"])),
            Command::Claude(args(&["update"]))
        );
    }

    #[test]
    fn test_classify_help_only_at_position_zero() {
        // A later `--help` is passthrough (it's an argument to claude, not
        // a request for cyolo help).
        assert_eq!(
            classify(&args(&["-p", "--help"])),
            Command::Claude(args(&["-p", "--help"]))
        );
    }
}
