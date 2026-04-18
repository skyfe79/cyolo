//! `cyolo profile whoami` — like `current`, plus the email from the
//! resolved profile's `.claude.json`. Useful to sanity-check which OAuth
//! account will be used before running `cyolo` for real.

use owo_colors::OwoColorize;

use crate::error::CyoloError;

use super::list::read_oauth_email;

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

            match read_oauth_email(&profile.config_dir) {
                Some(email) => println!("{} {}", "email:".bold(), email.green()),
                None => println!(
                    "{} {}",
                    "email:".bold(),
                    "(needs login — run `cyolo profile login <name>`)".yellow()
                ),
            }
        }
        None => {
            println!(
                "{}",
                "No profile detected. Using default Claude configuration (~/.claude).".dimmed()
            );
            if let Some(home) = dirs::home_dir() {
                if let Some(email) = read_oauth_email(&home.join(".claude")) {
                    println!("{} {}", "email:".bold(), email.green());
                }
            }
        }
    }
    Ok(())
}
