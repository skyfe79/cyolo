//! `cyolo profile rm` — unregister a profile from `~/.cyolo/config.json`.
//! The on-disk config_dir is preserved; the user can delete it themselves.

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Name of the profile to remove.
    pub name: String,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let mut cfg = CyoloConfig::load()?;

    let profile = cfg
        .profiles
        .get(&args.name)
        .ok_or_else(|| CyoloError::ProfileNotFound {
            name: args.name.clone(),
        })?;

    // Capture config_dir for the confirmation message before the remove call.
    let config_dir = profile.config_dir.clone();

    cfg.profiles.remove(&args.name);

    // Clear default when removing the profile that held that slot.
    if cfg.default.as_deref() == Some(args.name.as_str()) {
        cfg.default = None;
    }

    cfg.save()?;

    println!("Removed profile: {}", args.name.green());
    println!(
        "Directory preserved: {}",
        config_dir.display().to_string().green()
    );
    Ok(())
}
