//! `cyolo profile add` — register a new profile, optionally running
//! `claude /login` for it and seeding its User MCPs from `~/.claude.json`.

use std::path::PathBuf;

use owo_colors::OwoColorize;

use crate::config::{self, CyoloConfig, Profile};
use crate::error::CyoloError;
use crate::runner;
use crate::symlink;
use crate::util::expand_tilde;

use super::sync_mcp::report_mcp_sync;

/// CLI arguments for `cyolo profile add <name> [config-dir] [--no-share] [--no-login]`.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// Name to register the profile under.
    pub name: String,
    /// Config directory for the profile. Defaults to `~/.claude-<name>`.
    pub config_dir: Option<String>,
    /// Skip creating the six shared symlinks back into `~/.claude/`.
    #[arg(long)]
    pub no_share: bool,
    /// Skip auto-launching `claude` for `/login` after registration.
    #[arg(long)]
    pub no_login: bool,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    let Args {
        name,
        config_dir,
        no_share,
        no_login,
    } = args;

    // Resolve config_dir: use the provided path or default to ~/.claude-<name>.
    let config_dir: PathBuf = if let Some(dir) = config_dir {
        expand_tilde(&dir)
    } else {
        let home = dirs::home_dir().ok_or_else(|| CyoloError::ConfigIoError {
            context: "could not determine home directory".into(),
            source: std::io::Error::new(std::io::ErrorKind::NotFound, "home directory not found"),
        })?;
        home.join(format!(".claude-{name}"))
    };

    config::ensure_dir()?;
    let mut cfg = CyoloConfig::load()?;

    if cfg.profiles.contains_key(&name) {
        return Err(CyoloError::ProfileAlreadyExists { name: name.clone() });
    }

    // Create config_dir at 0700 if absent; reject if path exists as a non-directory.
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

    if !no_share {
        symlink::create_shared_symlinks(&config_dir)?;
    }

    cfg.profiles.insert(
        name.clone(),
        Profile {
            name: name.clone(),
            config_dir: config_dir.clone(),
        },
    );
    cfg.save()?;

    let symlink_note = if no_share {
        "(no shared symlinks)"
    } else if symlink::is_source_dir(&config_dir) {
        "(symlinks skipped, source directory)"
    } else {
        "(shared symlinks created)"
    };
    println!(
        "Added profile: {} -> {} {}",
        name.green(),
        config_dir.display().to_string().green(),
        symlink_note
    );

    // Auto-launch `claude /login` so the OAuth token lands in the Keychain
    // entry scoped to this profile's `CLAUDE_CONFIG_DIR`. Skipped when:
    //   * `--no-login`
    //   * `config_dir` resolves to `~/.claude` (the source directory — the
    //     default Keychain entry is already populated by prior usage).
    if !no_login && !symlink::is_source_dir(&config_dir) {
        println!();
        println!(
            "{} launching claude so you can run {} for this profile…",
            "→".cyan().bold(),
            "/login".bold()
        );
        println!(
            "{}",
            "  (skip this with --no-login on `cyolo profile add`)".dimmed()
        );
        runner::run_claude_login(&config_dir)?;
    }

    // Layer `mcpServers` onto whatever claude wrote into `.claude.json` (or
    // into an empty file for `--no-login`). Best-effort — a sync failure
    // must not make the whole `add` fail.
    report_mcp_sync(&config_dir);

    Ok(())
}
