//! `cyolo profile <...>` subcommand tree.
//!
//! Each subcommand lives in its own folder (`<cmd>/mod.rs` plus
//! `<cmd>/tests/`), and this file owns only the clap `Parser` +
//! `Subcommand` derives that glue the tree together. Routing goes
//! through [`dispatch`], which also handles the shape of clap's
//! `--help` / `--version` / parse-error responses.
//!
//! Keep this file thin: if something grows beyond routing, push it down
//! into the relevant subcommand module.

use clap::{CommandFactory, Parser, Subcommand};

use crate::error::CyoloError;

pub mod add;
pub mod current;
pub mod default_cmd;
pub mod init;
pub mod link;
pub mod list;
pub mod login;
pub mod marker;
pub mod picker;
pub mod rm;
pub mod sync_mcp;
pub mod whoami;

#[cfg(test)]
mod tests;

// Root clap struct for `cyolo profile <...>`. The implementation notes
// (why `no_binary_name = true`, why `disable_version_flag = true`) are
// on the `dispatch` fn below — keeping them off the struct so clap's
// auto-generated help stays focused on the user-facing `about` text.
#[derive(Parser, Debug)]
#[command(
    name = "profile",
    about = "Manage per-account config directories",
    no_binary_name = true,
    disable_version_flag = true
)]
pub struct ProfileCli {
    #[command(subcommand)]
    pub command: Option<ProfileCommand>,
}

#[derive(Subcommand, Debug)]
pub enum ProfileCommand {
    /// Register a new profile (auto-runs claude /login + seeds User MCPs).
    Add(add::Args),
    /// Remove a profile from ~/.cyolo/config.json (directory preserved).
    #[command(alias = "remove")]
    Rm(rm::Args),
    /// List all profiles with email + login state.
    #[command(alias = "ls")]
    List,
    /// Re-create the six shared symlinks for an already-registered profile.
    Link(link::Args),
    /// Re-run claude /login for a registered profile.
    Login(login::Args),
    /// Show the currently active profile (does not launch claude).
    Current,
    /// Show active profile + email from its .claude.json.
    Whoami,
    /// Create .claude-profile.json in the current directory.
    Init(init::Args),
    /// Get, set, or clear the default profile.
    Default(default_cmd::Args),
    /// Copy mcpServers from ~/.claude.json into a profile's .claude.json.
    #[command(name = "sync-mcp")]
    SyncMcp(sync_mcp::Args),
}

/// Parse `args` (the argv slice *after* "profile"), dispatch, and return.
///
/// `dispatch` intentionally does **not** call `e.exit()` on clap errors.
/// `--help` / `--version` / "missing subcommand" are success (Ok(())),
/// everything else becomes `Err(NonZeroExit(exit_code))` so tests can
/// observe it without process termination.
///
/// `no_binary_name = true` on [`ProfileCli`] means clap treats the
/// iterator as pure positional args, so we pass `args` through verbatim
/// without a synthetic program name.
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    let cli = match ProfileCli::try_parse_from(args) {
        Ok(c) => c,
        Err(e) => return handle_clap_error(e),
    };

    match cli.command {
        None => {
            // Bare `cyolo profile` → print full help (matches `--help`).
            ProfileCli::command().print_help().ok();
            println!();
            Ok(())
        }
        Some(ProfileCommand::Add(a)) => add::run(a),
        Some(ProfileCommand::Rm(a)) => rm::run(a),
        Some(ProfileCommand::List) => list::run(),
        Some(ProfileCommand::Link(a)) => link::run(a),
        Some(ProfileCommand::Login(a)) => login::run(a),
        Some(ProfileCommand::Current) => current::run(),
        Some(ProfileCommand::Whoami) => whoami::run(),
        Some(ProfileCommand::Init(a)) => init::run(a),
        Some(ProfileCommand::Default(a)) => default_cmd::run(a),
        Some(ProfileCommand::SyncMcp(a)) => sync_mcp::run(a),
    }
}

/// Turn a `clap::Error` into either `Ok(())` (help/version display) or
/// `Err(NonZeroExit(exit_code))` (genuine parse failure).
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
