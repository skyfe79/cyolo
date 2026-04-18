//! `cyolo profile link` — idempotently (re)create the six shared symlinks
//! for an already-registered profile.
//!
//! Use after a skill/command/plugin install into `~/.claude/` whose target
//! didn't exist when the profile was first registered, or after any manual
//! edit that broke the symlink farm.

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;
use crate::symlink;
use crate::util::expand_tilde;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Name of the registered profile to re-link.
    pub name: String,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    let profile = cfg
        .profiles
        .get(&args.name)
        .ok_or_else(|| CyoloError::ProfileNotFound {
            name: args.name.clone(),
        })?;

    // Normalize in case the on-disk config_dir was hand-edited with a tilde.
    let config_dir = expand_tilde(&profile.config_dir.to_string_lossy());

    symlink::create_shared_symlinks(&config_dir)?;

    println!("Symlinks updated for profile '{}'", args.name.green());
    Ok(())
}
