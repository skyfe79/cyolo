//! `cyolo profile default [name | --unset]` — read / write / clear the
//! `default` field of `~/.cyolo/config.json`.  The default fires during
//! walk-up resolution when no `.claude-profile.json` is found in cwd ↑.

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig};
use crate::error::CyoloError;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Registered profile name to set as default. Omit to print the current
    /// default. Conflicts with `--unset`.
    #[arg(conflicts_with = "unset")]
    pub name: Option<String>,
    /// Clear the default profile entirely.
    #[arg(long)]
    pub unset: bool,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    config::ensure_dir()?;

    if args.unset {
        let mut cfg = CyoloConfig::load()?;
        cfg.default = None;
        cfg.save()?;
        println!("Default profile cleared.");
        return Ok(());
    }

    match args.name {
        None => {
            // Read-only: print the current default, if any.
            let cfg = CyoloConfig::load()?;
            match &cfg.default {
                Some(name) => println!("Default profile: {}", name.green()),
                None => println!("{}", "No default profile set.".dimmed()),
            }
            Ok(())
        }
        Some(name) => {
            let mut cfg = CyoloConfig::load()?;
            if !cfg.profiles.contains_key(&name) {
                return Err(CyoloError::ProfileNotFound { name });
            }
            cfg.default = Some(name.clone());
            cfg.save()?;
            println!("Default profile set to: {}", name.green());
            Ok(())
        }
    }
}
