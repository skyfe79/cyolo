//! `cyolo profile set-env` — update Anthropic env-var overrides for a registered profile.
//!
//! Allows setting or unsetting `ANTHROPIC_BASE_URL`, `ANTHROPIC_API_KEY`, and
//! `ANTHROPIC_MODEL` on an existing profile without re-registering it.

use owo_colors::OwoColorize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;

#[derive(clap::Args, Debug)]
pub struct Args {
    /// Name of the registered profile to update.
    pub name: String,
    /// Set custom API base URL (e.g. https://api.deepseek.com).
    #[arg(long)]
    pub base_url: Option<String>,
    /// Set API key for the provider. Stored in ~/.cyolo/config.json only.
    #[arg(long)]
    pub api_key: Option<String>,
    /// Set model name override (e.g. deepseek-chat, deepseek-reasoner).
    #[arg(long)]
    pub model: Option<String>,
    /// Clear the base URL override.
    #[arg(long)]
    pub unset_base_url: bool,
    /// Clear the API key override.
    #[arg(long)]
    pub unset_api_key: bool,
    /// Clear the model override.
    #[arg(long)]
    pub unset_model: bool,
}

pub fn run(args: Args) -> Result<(), CyoloError> {
    let Args {
        name,
        base_url,
        api_key,
        model,
        unset_base_url,
        unset_api_key,
        unset_model,
    } = args;

    let mut cfg = CyoloConfig::load()?;
    let profile = cfg
        .profiles
        .get_mut(&name)
        .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;

    let mut changed = false;

    if unset_base_url {
        if profile.anthropic_base_url.take().is_some() {
            println!("  {} base_url: {}", "−".red().bold(), "unset".dimmed());
            changed = true;
        }
    } else if let Some(url) = base_url {
        let old = profile.anthropic_base_url.replace(url.clone());
        print_change("base_url", old.as_deref(), &url);
        changed = true;
    }

    if unset_api_key {
        if profile.anthropic_api_key.take().is_some() {
            println!("  {} api_key: {}", "−".red().bold(), "unset".dimmed());
            changed = true;
        }
    } else if let Some(key) = api_key {
        let old = profile.anthropic_api_key.replace(key.clone());
        print_change("api_key", old.as_deref(), "***");
        changed = true;
    }

    if unset_model {
        if profile.anthropic_model.take().is_some() {
            println!("  {} model: {}", "−".red().bold(), "unset".dimmed());
            changed = true;
        }
    } else if let Some(m) = model {
        let old = profile.anthropic_model.replace(m.clone());
        print_change("model", old.as_deref(), &m);
        changed = true;
    }

    if changed {
        cfg.save()?;
        println!("Updated profile: {}", name.green());
    } else {
        println!("No changes for profile: {}", name.dimmed());
    }

    Ok(())
}

fn print_change(field: &str, old: Option<&str>, new: &str) {
    match old {
        Some(prev) if prev == new => {}
        Some(prev) => println!(
            "  {} {}: {} → {}",
            "~".yellow().bold(),
            field,
            prev.dimmed(),
            new.green()
        ),
        None => println!("  {} {}: {}", "+".green().bold(), field, new.green()),
    }
}
