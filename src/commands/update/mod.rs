//! `cyolo update <version>` — switch the active Claude Code version by
//! repointing the launcher symlink at an already-installed build.
//!
//! Scope (deliberately narrow): this verb only switches between versions that
//! are *already downloaded* under the native install's `versions/` directory.
//! It never downloads. Bare `cyolo update` (no version) is NOT routed here —
//! the top-level classifier forwards it to `claude update` (Claude Code's own
//! auto-updater) to preserve the documented pass-through behavior.

use owo_colors::OwoColorize;

use crate::commands::native::{self, Install};
use crate::error::CyoloError;

/// Route `cyolo update <version>`.
///
/// The classifier dispatches here for `--help`/`-h` (→ cyolo's own help) and
/// for a positional `<version>` (→ switch). `args[0]` is therefore either a
/// help flag or the requested version; we still guard the empty case
/// defensively.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let version = match args.first().map(|s| s.as_str()) {
        Some("--help") | Some("-h") => {
            print_help();
            return Ok(());
        }
        Some(v) => v,
        None => {
            eprintln!(
                "{} usage: cyolo update <version>   (see `cyolo version ls`)",
                "error:".red().bold()
            );
            return Err(CyoloError::NonZeroExit(2));
        }
    };

    let install = native::discover()?;
    let installed = native::installed_versions_in(&install.versions_dir)?;

    if !installed.iter().any(|v| v == version) {
        print_not_installed(version, &installed, &install);
        return Err(CyoloError::NonZeroExit(1));
    }

    if install.current.as_deref() == Some(version) {
        println!("{} already on {}", "✓".green(), version.green().bold());
        return Ok(());
    }

    native::switch_in(&install.bin_link, &install.versions_dir, version)?;

    println!(
        "{} switched to {}",
        "✓".green().bold(),
        version.green().bold()
    );
    println!(
        "  {} → {}",
        install.bin_link.display().to_string().dimmed(),
        install.versions_dir.join(version).display().to_string().dimmed()
    );
    println!(
        "{}",
        "Run `claude --version` in a new shell to confirm.".dimmed()
    );
    Ok(())
}

/// Print cyolo's `update` help. Documents both halves of the verb: the local
/// switch (`update <version>`) and the pass-through (bare `update` →
/// `claude update`), since the classifier routes those two differently.
fn print_help() {
    println!(
        "{} — switch the active Claude Code version (already-installed builds only)",
        "cyolo update".bold()
    );
    println!();
    println!("{}", "Usage:".bold());
    println!("  cyolo update <version>     Repoint the launcher symlink at an installed version");
    println!("  cyolo update --help        Show this message");
    println!();
    println!("{}", "Behavior:".bold());
    println!("  {}", "* Switches between builds already under ~/.local/share/claude/versions");
    println!("  {}", "  (atomic symlink repoint — no download). If the version isn't installed,");
    println!("  {}", "  it prints the installed list and does nothing.");
    println!(
        "  {}",
        "* Bare `cyolo update` (no version) passes through to `claude update` —"
    );
    println!("  {}", "  Claude Code's native auto-updater, which fetches the latest build.");
    println!();
    println!("{}", "See also:".bold());
    println!("  cyolo version ls           List installed versions");
    println!("  cyolo version ls remote    List upstream releases (npm registry)");
}

/// Explain that the requested version isn't present and point at the things
/// that *do* fetch versions — `cyolo` itself only switches between builds that
/// are already on disk, so a not-found path must hand the user off rather than
/// pretend to install.
fn print_not_installed(version: &str, installed: &[String], install: &Install) {
    eprintln!(
        "{} version {} is not installed",
        "error:".red().bold(),
        version.bold()
    );
    if installed.is_empty() {
        eprintln!(
            "  No versions found under {}",
            install.versions_dir.display()
        );
    } else {
        eprintln!("  Installed: {}", installed.join(", "));
    }
    eprintln!();
    eprintln!("  cyolo only switches between already-downloaded versions.");
    eprintln!(
        "  To fetch a newer build, run {} (Claude Code's native updater),",
        "`cyolo update`".bold()
    );
    eprintln!("  then re-run {}.", format!("`cyolo update {version}`").bold());
    eprintln!(
        "  Browse upstream releases with {}.",
        "`cyolo version ls remote`".bold()
    );
}
