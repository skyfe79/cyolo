use std::path::PathBuf;

use crate::config::{self, CyoloConfig, Profile};
use crate::error::CyoloError;

/// Add a new profile to the config.
///
/// Usage: `cyolo profile add <name> [config-dir]`
#[allow(dead_code)]
pub fn add(args: &[String]) -> Result<(), CyoloError> {
    let name = args.first().ok_or_else(|| {
        eprintln!("Usage: cyolo profile add <name> [config-dir]");
        CyoloError::NonZeroExit(1)
    })?;

    // Resolve config_dir: use provided path or default to ~/.claude-<name>
    let config_dir = if let Some(dir) = args.get(1) {
        expand_tilde(dir)
    } else {
        let home = dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
            context: "could not determine home directory".into(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
        })?;
        home.join(format!(".claude-{name}"))
    };

    // Ensure ~/.cyolo/ exists
    config::ensure_dir()?;

    // Load config
    let mut cfg = CyoloConfig::load()?;

    // Check for duplicate
    if cfg.profiles.contains_key(name) {
        return Err(CyoloError::ProfileAlreadyExists { name: name.clone() });
    }

    // Create config_dir with 0700 if it doesn't exist; reject if path exists but is not a directory
    if config_dir.exists() {
        if !config_dir.is_dir() {
            return Err(CyoloError::ConfigIoError {
                context: format!("{} exists but is not a directory", config_dir.display()),
                source: std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    "path is not a directory",
                ),
            });
        }
    } else {
        use std::os::unix::fs::DirBuilderExt;

        std::fs::DirBuilder::new()
            .mode(0o700)
            .recursive(true)
            .create(&config_dir)
            .map_err(|e| CyoloError::ConfigIoError {
                context: format!("failed to create directory {}", config_dir.display()),
                source: e,
            })?;
    }

    // Register profile
    cfg.profiles.insert(
        name.clone(),
        Profile {
            name: name.clone(),
            config_dir: config_dir.clone(),
        },
    );

    // Save config
    cfg.save()?;

    println!("Added profile: {} -> {}", name, config_dir.display());
    Ok(())
}

/// Expand leading `~` or `~/` to the user's home directory.
fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        dirs::home_dir().unwrap_or_else(|| PathBuf::from("~"))
    } else if let Some(rest) = path.strip_prefix("~/") {
        match dirs::home_dir() {
            Some(home) => home.join(rest),
            None => PathBuf::from(path),
        }
    } else {
        PathBuf::from(path)
    }
}
