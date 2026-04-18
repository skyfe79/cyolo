//! `cyolo profile init` — write `.claude-profile.json` into the current
//! working directory so walk-up resolution binds this tree to a profile.
//!
//! Resolution order:
//!   1. `name` argument given → use it directly.
//!   2. No arg + default profile set → use the default.
//!   3. No arg + no default + TTY → interactive picker fallback.
//!   4. No arg + no default + non-TTY → usage error (safe for CI / scripts).

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;
use crate::util::is_interactive;

use super::marker::write_profile_marker;
use super::picker::interactive_init_menu;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Registered profile name to bind. Omit to use the default profile or
    /// to open the interactive picker (TTY only).
    pub name: Option<String>,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    let name = match args.name {
        Some(n) => n,
        None => match &cfg.default {
            Some(default_name) => default_name.clone(),
            None => {
                if is_interactive() {
                    // The picker handles marker writing itself. `profile init`
                    // only needs to know the picker returned cleanly.
                    interactive_init_menu()?;
                    return Ok(());
                }
                eprintln!(
                    "{} no profile name given and no default profile set",
                    "error:".red().bold()
                );
                eprintln!(
                    "{} cyolo profile init <name>",
                    "Usage:".yellow().bold()
                );
                return Err(CyoloError::NonZeroExit(1));
            }
        },
    };

    if !cfg.profiles.contains_key(&name) {
        return Err(CyoloError::ProfileNotFound { name });
    }

    write_profile_marker(&name)
}
