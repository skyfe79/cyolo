use owo_colors::OwoColorize;

use crate::commands::diet;
use crate::commands::profile;
use crate::commands::profile::picker;
use crate::commands::update;
use crate::commands::use_cmd;
use crate::commands::version;
use crate::detect;
use crate::error::CyoloError;
use crate::runner;
use crate::util;

/// Top-level command classification.
///
/// cyolo-specific subcommands (`profile`, `diet`, `version`, `use <ver>`,
/// `update <ver>`, `help`) are handled in process. Everything else is
/// forwarded verbatim to `claude --dangerously-skip-permissions`.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// `cyolo profile <...>` → profile management
    Profile(Vec<String>),
    /// `cyolo diet <...>` → config cleanup
    Diet(Vec<String>),
    /// `cyolo version <...>` → list installed / upstream Claude Code versions
    Version(Vec<String>),
    /// `cyolo use <version>` → switch the active version, downloading it first
    /// if it isn't installed yet
    Use(Vec<String>),
    /// `cyolo update <version>` → legacy form; now redirects to `cyolo use`
    Update(Vec<String>),
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
        Some("version") => Command::Version(args[1..].to_vec()),
        // `use` is cyolo's version-switch verb: switch to a build, downloading
        // it first if it isn't installed yet. Everything after the token is
        // handed to the verb (version + flags like `--yes`).
        Some("use") => Command::Use(args[1..].to_vec()),
        // `update` now means "update to the latest build" — it passes through
        // to `claude update` (Claude Code's native auto-updater). The one
        // exception is a leftover positional `<version>` (the old switch
        // habit): we intercept it to redirect the user to `cyolo use`.
        Some("update") => match args.get(1).map(|s| s.as_str()) {
            Some(a) if !a.starts_with('-') => Command::Update(args[1..].to_vec()),
            _ => Command::Claude(args.to_vec()),
        },
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
        Command::Version(args) => version::dispatch(&args),
        Command::Use(args) => use_cmd::dispatch(&args),
        Command::Update(args) => update::dispatch(&args),
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
                resolved.is_none() && args.is_empty() && util::is_interactive();
            if show_picker {
                match picker::interactive_init_menu() {
                    Ok(picker::PickerOutcome::MarkerWritten) => {
                        // Re-resolve so the brand-new marker is honoured on
                        // this very same invocation — no second `cyolo` call.
                        resolved = detect::resolve_profile()?;
                    }
                    Ok(picker::PickerOutcome::NewProfileRegistered) => {
                        // `add` already ran `claude /login` interactively.
                        // Launching another claude session here would surprise
                        // the user with a back-to-back double launch — bail.
                        return Ok(());
                    }
                    Ok(picker::PickerOutcome::Quit) => {
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
    println!("  {}{}  Manage per-account config directories", "profile".green(), " ...        ");
    println!("  {}{}  Report / reclaim orphaned project data + caches", "diet".green(), " ...           ");
    println!("  {}{}  List installed (and upstream) Claude Code versions", "version".green(), " [ls [remote]]");
    println!("  {}{}  Switch the active version (downloads it first if missing)", "use".green(), " <ver>        ");
    println!("  {}{}  Update to the latest build (passes through to claude update)", "update".green(), "           ");
    println!("  {}{}  Show this message (also: --help / -h)", "help".green(), "               ");
    println!();
    println!("{}", "Anything else is forwarded as:".bold());
    println!("  claude --dangerously-skip-permissions <args...>");
    println!();
    println!("{}", "Examples:".bold());
    println!("  cyolo                          # interactive claude with resolved profile");
    println!("  cyolo -p \"hi there\"            # one-shot prompt via claude -p");
    println!("  cyolo profile                  # detailed profile subcommand help");
    println!("  cyolo diet                     # dry-run cleanup report");
    println!("  cyolo version ls               # list installed versions");
    println!("  cyolo version ls remote        # list upstream releases (npm registry)");
    println!("  cyolo use 2.1.158              # switch to 2.1.158, downloading it if needed");
    println!("  cyolo update                   # update to the latest build (claude update)");
    println!();
    println!("{}", "Notes:".dimmed());
    println!("  {}", "* `cyolo use <ver>` switches between native-install builds under".dimmed());
    println!("  {}", "  ~/.local/share/claude/versions — instant if installed, else it fetches".dimmed());
    println!("  {}", "  the build via `claude install <ver>` first.".dimmed());
    println!("  {}", "* `cyolo update` passes through to `claude update` (fetches the latest).".dimmed());
    println!("  {}", "* `cyolo --version` prints claude's version (pass-through).".dimmed());
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
mod tests;
