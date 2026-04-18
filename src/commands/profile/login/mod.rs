//! `cyolo profile login` — launch `claude` with the profile's
//! `CLAUDE_CONFIG_DIR` so the user can run `/login` inside and refresh or
//! swap the bound OAuth account.

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;
use crate::runner;
use crate::util::expand_tilde;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Name of the registered profile to re-login.
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

    let config_dir = expand_tilde(&profile.config_dir.to_string_lossy());

    println!(
        "{} launching claude for profile {} — run {} inside",
        "→".cyan().bold(),
        args.name.green(),
        "/login".bold()
    );
    runner::run_claude_login(&config_dir)
}
