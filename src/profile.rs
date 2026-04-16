use std::path::PathBuf;

use crate::config::{self, CyoloConfig, Profile};
use crate::error::CyoloError;
use crate::symlink;

/// Route profile subcommands.
///
/// Usage: `cyolo profile <add|rm|list>`
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    match args.first().map(|s| s.as_str()) {
        Some("add") => add(&args[1..]),
        Some("rm") | Some("remove") => rm(&args[1..]),
        Some("list") | Some("ls") => list(),
        Some("link") => link(&args[1..]),
        None => {
            println!("Usage: cyolo profile <add|rm|list|link>");
            println!();
            println!("Commands:");
            println!("  add <name> [config-dir] [--no-share]  Register a new profile");
            println!("  rm <name>                Remove a profile");
            println!("  list                     List all profiles");
            println!("  link <name>              Re-create shared symlinks for a profile");
            Ok(())
        }
        Some(cmd) => {
            eprintln!("cyolo: unknown profile command '{cmd}'");
            eprintln!("Available: add, rm, list, link");
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

/// Add a new profile to the config.
///
/// Usage: `cyolo profile add <name> [config-dir] [--no-share]`
pub fn add(args: &[String]) -> Result<(), CyoloError> {
    // Parse --no-share flag (position-independent)
    let no_share = args.iter().any(|a| a == "--no-share");
    let positional: Vec<&String> = args.iter().filter(|a| a.as_str() != "--no-share").collect();

    let name = positional.first().ok_or_else(|| {
        eprintln!("Usage: cyolo profile add <name> [config-dir] [--no-share]");
        CyoloError::NonZeroExit(1)
    })?;

    // Resolve config_dir: use provided path or default to ~/.claude-<name>
    let config_dir = if let Some(dir) = positional.get(1) {
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
    if cfg.profiles.contains_key(*name) {
        return Err(CyoloError::ProfileAlreadyExists { name: (*name).clone() });
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

    // Create shared symlinks unless --no-share
    if !no_share {
        symlink::create_shared_symlinks(&config_dir)?;
    }

    // Register profile
    cfg.profiles.insert(
        (*name).clone(),
        Profile {
            name: (*name).clone(),
            config_dir: config_dir.clone(),
        },
    );

    // Save config
    cfg.save()?;

    // Confirmation message with symlink status
    let symlink_note = if no_share {
        "(no shared symlinks)"
    } else if symlink::is_source_dir(&config_dir) {
        "(symlinks skipped, source directory)"
    } else {
        "(shared symlinks created)"
    };
    println!("Added profile: {} -> {} {}", name, config_dir.display(), symlink_note);
    Ok(())
}

/// Remove a profile from the config.
///
/// The profile's directory on disk is preserved (not deleted).
///
/// Usage: `cyolo profile rm <name>`
pub fn rm(args: &[String]) -> Result<(), CyoloError> {
    let name = args.first().ok_or_else(|| {
        eprintln!("Usage: cyolo profile rm <name>");
        CyoloError::NonZeroExit(1)
    })?;

    // Ensure ~/.cyolo/ exists
    config::ensure_dir()?;

    // Load config
    let mut cfg = CyoloConfig::load()?;

    // Check profile exists
    let profile = cfg
        .profiles
        .get(name)
        .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;

    // Capture config_dir for the confirmation message before removing
    let config_dir = profile.config_dir.clone();

    // Remove from profiles
    cfg.profiles.remove(name);

    // Clear default if removing the default profile
    if cfg.default.as_deref() == Some(name.as_str()) {
        cfg.default = None;
    }

    // Save config
    cfg.save()?;

    println!("Removed profile: {name}");
    println!("Directory preserved: {}", config_dir.display());
    Ok(())
}

/// List all registered profiles.
///
/// Displays profiles in a sorted table with the default profile
/// marked by a `*` prefix.
///
/// Usage: `cyolo profile list`
pub fn list() -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    if cfg.profiles.is_empty() {
        println!("No profiles registered. Run: cyolo profile add <name>");
        return Ok(());
    }

    let max_width = cfg.profiles.keys().map(|k| k.len()).max().unwrap_or(0);

    for (name, profile) in &cfg.profiles {
        let marker = if cfg.default.as_deref() == Some(name.as_str()) {
            "* "
        } else {
            "  "
        };
        let dir = profile.config_dir.display();
        println!("{marker}{name:<max_width$} -> {dir}");
    }

    Ok(())
}

/// Re-create shared symlinks for an already-registered profile.
///
/// Idempotent: existing correct symlinks are left as-is.
///
/// Usage: `cyolo profile link <name>`
pub fn link(args: &[String]) -> Result<(), CyoloError> {
    if args.len() != 1 {
        eprintln!("Usage: cyolo profile link <name>");
        return Err(CyoloError::NonZeroExit(1));
    }
    let name = &args[0];

    config::ensure_dir()?;

    let cfg = CyoloConfig::load()?;

    let profile = cfg
        .profiles
        .get(name)
        .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;

    // Normalize config_dir in case it was manually edited with a tilde prefix.
    let config_dir = expand_tilde(&profile.config_dir.to_string_lossy());

    symlink::create_shared_symlinks(&config_dir)?;

    println!("Symlinks updated for profile '{name}'");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_dispatch_unknown_subcommand_returns_error() {
        let result = dispatch(&args(&["unknown"]));
        assert!(result.is_err());
    }
}
