//! `cyolo profile wizard` — guided interactive profile creation.
//!
//! Asks the user step-by-step questions and delegates the actual registration
//! to `profile::add::run`, keeping all validation and persistence logic in one place.

use std::io::Write;

use owo_colors::OwoColorize;

use crate::config::CyoloConfig;
use crate::error::CyoloError;
use crate::util::{is_interactive, read_line_trimmed};

use super::add;

const DEEPSEEK_BASE_URL: &str = "https://api.deepseek.com";
const DEEPSEEK_DEFAULT_MODEL: &str = "deepseek-chat";

enum ProviderChoice {
    None,
    DeepSeek,
    Custom,
}

pub fn run() -> Result<(), CyoloError> {
    if !is_interactive() {
        return Err(CyoloError::ConfigIoError {
            context: "`cyolo profile wizard` requires an interactive terminal (TTY)".into(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                "stdin or stdout is not a terminal",
            ),
        });
    }

    println!(
        "{} {}",
        "cyolo profile wizard".cyan().bold(),
        "— interactive profile setup".dimmed()
    );
    println!();

    let cfg = CyoloConfig::load()?;

    // Step 1: name
    let name = prompt_name(&cfg)?;

    // Step 2: config dir
    let config_dir = prompt_config_dir(&name)?;

    // Step 3: shared symlinks
    let no_share = !prompt_yn("Create shared symlinks?", true)?;

    // Step 4: provider
    let provider = prompt_provider()?;

    // Step 5: provider-specific fields
    let (base_url, api_key, model) = match provider {
        ProviderChoice::None => (None, None, None),
        ProviderChoice::DeepSeek => prompt_deepseek_fields()?,
        ProviderChoice::Custom => prompt_custom_fields()?,
    };

    // Step 6: login — default No when a provider is configured
    let default_login = base_url.is_none();
    let no_login = !prompt_yn("Run /login for this profile?", default_login)?;

    // Step 7: summary + confirm
    println!();
    print_summary(&name, &config_dir, no_share, no_login, &base_url, &api_key, &model);
    println!();

    if !prompt_yn("Add this profile?", true)? {
        println!("{}", "Aborted.".dimmed());
        return Ok(());
    }

    println!();
    add::run(add::Args {
        name,
        config_dir,
        no_share,
        no_login,
        base_url,
        api_key,
        model,
    })
}

fn prompt_name(cfg: &CyoloConfig) -> Result<String, CyoloError> {
    loop {
        print!("{} ", "Profile name:".bold());
        std::io::stdout().flush().ok();
        let input = read_line_trimmed()?;

        if input.is_empty() {
            eprintln!("{} name cannot be empty", "error:".red().bold());
            continue;
        }
        if input.contains('/') || input.contains('\\') {
            eprintln!("{} name must not contain path separators", "error:".red().bold());
            continue;
        }
        if cfg.profiles.contains_key(&input) {
            eprintln!(
                "{} profile '{}' already exists",
                "error:".red().bold(),
                input
            );
            continue;
        }
        return Ok(input);
    }
}

fn prompt_config_dir(name: &str) -> Result<Option<String>, CyoloError> {
    let default_hint = format!("~/.claude-{name}");
    print!(
        "{} [{}]: ",
        "Config directory".bold(),
        default_hint.dimmed()
    );
    std::io::stdout().flush().ok();
    let input = read_line_trimmed()?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

fn prompt_yn(question: &str, default_yes: bool) -> Result<bool, CyoloError> {
    let hint = if default_yes { "[Y/n]" } else { "[y/N]" };
    loop {
        print!("{} {}: ", question.bold(), hint.dimmed());
        std::io::stdout().flush().ok();
        let input = read_line_trimmed()?;
        match input.to_lowercase().as_str() {
            "" => return Ok(default_yes),
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("{} please enter y or n", "error:".red().bold()),
        }
    }
}

fn prompt_provider() -> Result<ProviderChoice, CyoloError> {
    println!();
    println!("{}:", "Provider".bold());
    println!("  {}. None (standard Anthropic)", "1".cyan());
    println!("  {}. DeepSeek", "2".cyan());
    println!("  {}. Custom", "3".cyan());

    loop {
        print!("{} [1]: ", "Selection".bold());
        std::io::stdout().flush().ok();
        let input = read_line_trimmed()?;
        match input.trim() {
            "" | "1" => return Ok(ProviderChoice::None),
            "2" => return Ok(ProviderChoice::DeepSeek),
            "3" => return Ok(ProviderChoice::Custom),
            _ => eprintln!("{} please enter 1, 2, or 3", "error:".red().bold()),
        }
    }
}

fn prompt_deepseek_fields(
) -> Result<(Option<String>, Option<String>, Option<String>), CyoloError> {
    println!();

    // Base URL (preset as default, but user can override)
    print!(
        "  {} [{}]: ",
        "Base URL".bold(),
        DEEPSEEK_BASE_URL.dimmed()
    );
    std::io::stdout().flush().ok();
    let base_url_input = read_line_trimmed()?;
    let base_url = if base_url_input.is_empty() {
        DEEPSEEK_BASE_URL.to_string()
    } else {
        base_url_input
    };

    // API key
    print!(
        "  {} {}: ",
        "API key".bold(),
        "(stored only in ~/.cyolo/config.json)".dimmed()
    );
    std::io::stdout().flush().ok();
    let api_key_input = read_line_trimmed()?;
    let api_key = if api_key_input.is_empty() { None } else { Some(api_key_input) };

    // Model
    print!(
        "  {} [{}]: ",
        "Model".bold(),
        DEEPSEEK_DEFAULT_MODEL.dimmed()
    );
    std::io::stdout().flush().ok();
    let model_input = read_line_trimmed()?;
    let model = if model_input.is_empty() {
        Some(DEEPSEEK_DEFAULT_MODEL.to_string())
    } else {
        Some(model_input)
    };

    Ok((Some(base_url), api_key, model))
}

fn prompt_custom_fields(
) -> Result<(Option<String>, Option<String>, Option<String>), CyoloError> {
    println!();

    // Base URL
    print!("  {}: ", "Base URL".bold());
    std::io::stdout().flush().ok();
    let base_url_input = read_line_trimmed()?;
    let base_url = if base_url_input.is_empty() { None } else { Some(base_url_input) };

    // API key
    print!(
        "  {} {}: ",
        "API key".bold(),
        "(stored only in ~/.cyolo/config.json)".dimmed()
    );
    std::io::stdout().flush().ok();
    let api_key_input = read_line_trimmed()?;
    let api_key = if api_key_input.is_empty() { None } else { Some(api_key_input) };

    // Model
    print!("  {}: ", "Model".bold());
    std::io::stdout().flush().ok();
    let model_input = read_line_trimmed()?;
    let model = if model_input.is_empty() { None } else { Some(model_input) };

    Ok((base_url, api_key, model))
}

fn print_summary(
    name: &str,
    config_dir: &Option<String>,
    no_share: bool,
    no_login: bool,
    base_url: &Option<String>,
    api_key: &Option<String>,
    model: &Option<String>,
) {
    let sep = "─".repeat(40);
    println!("{}", sep.dimmed());

    let default_dir = format!("~/.claude-{name}");
    let dir_display = config_dir.as_deref().unwrap_or(&default_dir);

    println!("  {}  {}", "profile:".bold(), name.green());
    println!("  {}      {}", "dir:".bold(), dir_display.green());
    println!(
        "  {}  {}",
        "symlinks:".bold(),
        if no_share { "no".dimmed().to_string() } else { "yes".green().to_string() }
    );
    println!(
        "  {}    {}",
        "login:".bold(),
        if no_login { "no".dimmed().to_string() } else { "yes".green().to_string() }
    );

    if let Some(url) = base_url {
        println!("  {}  {}", "base_url:".bold(), url.green());
    }
    if api_key.is_some() {
        println!("  {}  {}", "api_key:".bold(), "***".dimmed());
    }
    if let Some(m) = model {
        println!("  {}    {}", "model:".bold(), m.green());
    }

    println!("{}", sep.dimmed());
}
