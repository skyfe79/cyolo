use std::path::PathBuf;

use crate::config::{self, CyoloConfig, Profile};
use crate::error::CyoloError;
use crate::symlink;

/// Route profile subcommands.
///
/// Usage: `cyolo profile <add|rm|list|link|current|init|default>`
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    match args.first().map(|s| s.as_str()) {
        Some("add") => add(&args[1..]),
        Some("rm") | Some("remove") => rm(&args[1..]),
        Some("list") | Some("ls") => list(),
        Some("link") => link(&args[1..]),
        Some("current") => current(&args[1..]),
        Some("init") => profile_init(&args[1..]),
        Some("default") => profile_default(&args[1..]),
        None => {
            println!("Usage: cyolo profile <add|rm|list|link|current|init|default>");
            println!();
            println!("Commands:");
            println!("  add <name> [config-dir] [--no-share]  Register a new profile");
            println!("  rm <name>                Remove a profile");
            println!("  list                     List all profiles");
            println!("  link <name>              Re-create shared symlinks for a profile");
            println!("  current                  Show the currently active profile");
            println!("  init [name]              Create .claude-profile.json in current directory");
            println!("  default [name|--unset]   Get/set/clear the default profile");
            Ok(())
        }
        Some(cmd) => {
            eprintln!("cyolo: unknown profile command '{cmd}'");
            eprintln!("Available: add, rm, list, link, current, init, default");
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

/// Show the currently active profile.
///
/// Runs `detect::resolve_profile()` and prints the result.
/// Does NOT launch claude.
///
/// Usage: `cyolo profile current`
pub fn current(args: &[String]) -> Result<(), CyoloError> {
    if !args.is_empty() {
        eprintln!("Usage: cyolo profile current");
        return Err(CyoloError::NonZeroExit(1));
    }
    let resolved = crate::detect::resolve_profile()?;
    match resolved {
        Some(profile) => {
            if let Some(name) = &profile.name {
                println!("profile: {name}");
            }
            println!("config_dir: {}", profile.config_dir.display());
            println!("source: {}", profile.source);
        }
        None => {
            println!("No profile detected. Using default Claude configuration (~/.claude).");
        }
    }
    Ok(())
}

/// Get, set, or clear the default profile.
///
/// - No args: print the current default profile name.
/// - One arg (name): validate and set the default profile.
/// - `--unset`: clear the default profile.
///
/// Usage: `cyolo profile default [name | --unset]`
pub fn profile_default(args: &[String]) -> Result<(), CyoloError> {
    config::ensure_dir()?;

    match args.len() {
        0 => {
            let cfg = CyoloConfig::load()?;
            match &cfg.default {
                Some(name) => println!("Default profile: {name}"),
                None => println!("No default profile set."),
            }
            Ok(())
        }
        1 => {
            if args[0] == "--unset" {
                let mut cfg = CyoloConfig::load()?;
                cfg.default = None;
                cfg.save()?;
                println!("Default profile cleared.");
                Ok(())
            } else {
                let name = &args[0];
                let mut cfg = CyoloConfig::load()?;
                if !cfg.profiles.contains_key(name) {
                    return Err(CyoloError::ProfileNotFound { name: name.clone() });
                }
                cfg.default = Some(name.clone());
                cfg.save()?;
                println!("Default profile set to: {name}");
                Ok(())
            }
        }
        _ => {
            eprintln!("Usage: cyolo profile default [name | --unset]");
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

/// Create `.claude-profile.json` in the current working directory.
///
/// Resolves the profile name from the first positional argument, or
/// falls back to `config.default`.  Validates that the name is
/// registered before writing.  Refuses to overwrite an existing file.
///
/// Usage: `cyolo profile init [name]`
pub fn profile_init(args: &[String]) -> Result<(), CyoloError> {
    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    // Resolve profile name
    let name = match args.len() {
        0 => match &cfg.default {
            Some(default_name) => default_name.clone(),
            None => {
                eprintln!("No profile name given and no default profile set.");
                eprintln!("Usage: cyolo profile init <name>");
                return Err(CyoloError::NonZeroExit(1));
            }
        },
        1 => args[0].clone(),
        _ => {
            eprintln!("Usage: cyolo profile init <name>");
            return Err(CyoloError::NonZeroExit(1));
        }
    };

    // Validate the name exists in config.profiles
    if !cfg.profiles.contains_key(&name) {
        return Err(CyoloError::ProfileNotFound { name });
    }

    // Check if .claude-profile.json already exists in cwd
    let cwd = std::env::current_dir().map_err(|e| CyoloError::ConfigIoError {
        context: "could not determine current directory".into(),
        source: e,
    })?;
    let profile_path = cwd.join(".claude-profile.json");

    if profile_path.exists() {
        eprintln!(
            "cyolo: .claude-profile.json already exists in {}",
            cwd.display()
        );
        return Err(CyoloError::NonZeroExit(1));
    }

    // Write the file
    let contents = serde_json::to_string_pretty(&serde_json::json!({"name": name}))
        .expect("JSON serialization of simple object cannot fail");
    std::fs::write(&profile_path, format!("{contents}\n")).map_err(|e| {
        CyoloError::ConfigIoError {
            context: format!("failed to write {}", profile_path.display()),
            source: e,
        }
    })?;

    println!("Created .claude-profile.json (profile: {name})");
    Ok(())
}

/// Expand leading `~` or `~/` to the user's home directory.
pub(crate) fn expand_tilde(path: &str) -> PathBuf {
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

    #[test]
    fn test_current_rejects_extra_args() {
        let result = current(&args(&["unexpected"]));
        assert!(result.is_err());
    }
}
