//! `cyolo profile current` — print the profile that would be used from
//! the cwd right now. Does **not** launch `claude`.

use owo_colors::OwoColorize;

use crate::error::CyoloError;

pub fn run() -> Result<(), CyoloError> {
    let resolved = crate::detect::resolve_profile()?;
    match resolved {
        Some(profile) => {
            if let Some(name) = &profile.name {
                println!("{} {}", "profile:".bold(), name.green());
            }
            println!(
                "{} {}",
                "config_dir:".bold(),
                profile.config_dir.display().to_string().green()
            );
            println!("{} {}", "source:".bold(), profile.source.green());
        }
        None => {
            println!(
                "{}",
                "No profile detected. Using default Claude configuration (~/.claude).".dimmed()
            );
        }
    }
    Ok(())
}
