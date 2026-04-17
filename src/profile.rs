use std::path::PathBuf;

use crate::config::{self, CyoloConfig, Profile};
use crate::error::CyoloError;
use crate::runner;
use crate::symlink;
use owo_colors::OwoColorize;

/// Route profile subcommands.
///
/// Usage: `cyolo profile <add|rm|list|link|current|init|default>`
pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
    match args.first().map(|s| s.as_str()) {
        Some("add") => add(&args[1..]),
        Some("rm") | Some("remove") => rm(&args[1..]),
        Some("list") | Some("ls") => list(),
        Some("link") => link(&args[1..]),
        Some("login") => login(&args[1..]),
        Some("current") => current(&args[1..]),
        Some("whoami") => whoami(&args[1..]),
        Some("init") => profile_init(&args[1..]),
        Some("default") => profile_default(&args[1..]),
        None => {
            println!("{} cyolo profile <add|rm|list|link|login|current|whoami|init|default>", "Usage:".yellow().bold());
            println!();
            println!("Commands:");
            println!("  add <name> [config-dir] [--no-share] [--no-login]");
            println!("                           Register a new profile (auto-runs claude /login)");
            println!("  rm <name>                Remove a profile");
            println!("  list                     List all profiles with email + login state");
            println!("  link <name>              Re-create shared symlinks for a profile");
            println!("  login <name>             Re-run claude /login for a registered profile");
            println!("  current                  Show the currently active profile");
            println!("  whoami                   Show active profile + email from its .claude.json");
            println!("  init [name]              Create .claude-profile.json in current directory");
            println!("  default [name|--unset]   Get/set/clear the default profile");
            Ok(())
        }
        Some(cmd) => {
            eprintln!("{} unknown profile command '{}'", "error:".red().bold(), cmd.bold());
            eprintln!("{}", "Available: add, rm, list, link, login, current, whoami, init, default".dimmed());
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

/// Add a new profile to the config.
///
/// Usage: `cyolo profile add <name> [config-dir] [--no-share]`
pub fn add(args: &[String]) -> Result<(), CyoloError> {
    // Parse flags (position-independent)
    let no_share = args.iter().any(|a| a == "--no-share");
    let no_login = args.iter().any(|a| a == "--no-login");
    let positional: Vec<&String> = args
        .iter()
        .filter(|a| a.as_str() != "--no-share" && a.as_str() != "--no-login")
        .collect();

    let name = positional.first().ok_or_else(|| {
        eprintln!(
            "{} cyolo profile add <name> [config-dir] [--no-share] [--no-login]",
            "Usage:".yellow().bold()
        );
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
    println!("Added profile: {} -> {} {}", name.green(), config_dir.display().to_string().green(), symlink_note);

    // Auto-launch claude so the user can `/login` with the right OAuth account
    // for this profile. Each distinct CLAUDE_CONFIG_DIR lands in its own
    // Keychain entry (`Claude Code-credentials-<sha256[:8]>`), so the token
    // captured here is scoped to this profile. Skipped when:
    //   - user passes `--no-login`
    //   - config_dir resolves to `~/.claude` (the source directory — nothing to
    //     do because the default entry is already populated by prior usage)
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

    Ok(())
}

/// Remove a profile from the config.
///
/// The profile's directory on disk is preserved (not deleted).
///
/// Usage: `cyolo profile rm <name>`
pub fn rm(args: &[String]) -> Result<(), CyoloError> {
    let name = args.first().ok_or_else(|| {
        eprintln!("{} cyolo profile rm <name>", "Usage:".yellow().bold());
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

    println!("Removed profile: {}", name.green());
    println!("Directory preserved: {}", config_dir.display().to_string().green());
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
        println!("No profiles registered. {}", "Run: cyolo profile add <name>".dimmed());
        return Ok(());
    }

    let max_width = cfg.profiles.keys().map(|k| k.len()).max().unwrap_or(0);

    for (name, profile) in &cfg.profiles {
        let padded = format!("{name:<max_width$}");
        let dir = profile.config_dir.display();
        let status = match read_oauth_email(&profile.config_dir) {
            Some(email) => format!("{}", email.green()),
            None => format!("{}", "(needs login)".yellow()),
        };
        if cfg.default.as_deref() == Some(name.as_str()) {
            println!("{} {} -> {}  {}", "*".green().bold(), padded.bold(), dir, status);
        } else {
            println!("  {} -> {}  {}", padded.bold(), dir, status);
        }
    }

    Ok(())
}

/// Run `claude` with `CLAUDE_CONFIG_DIR` set to the profile's directory so the
/// user can `/login` (or re-login) with the OAuth account bound to that profile.
///
/// Usage: `cyolo profile login <name>`
pub fn login(args: &[String]) -> Result<(), CyoloError> {
    if args.len() != 1 {
        eprintln!("{} cyolo profile login <name>", "Usage:".yellow().bold());
        return Err(CyoloError::NonZeroExit(1));
    }
    let name = &args[0];

    config::ensure_dir()?;
    let cfg = CyoloConfig::load()?;

    let profile = cfg
        .profiles
        .get(name)
        .ok_or_else(|| CyoloError::ProfileNotFound { name: name.clone() })?;

    let config_dir = expand_tilde(&profile.config_dir.to_string_lossy());

    println!(
        "{} launching claude for profile {} — run {} inside",
        "→".cyan().bold(),
        name.green(),
        "/login".bold()
    );
    runner::run_claude_login(&config_dir)
}

/// Show the active profile plus the email address from its `.claude.json`.
///
/// Unlike `current`, this reads the resolved profile's `.claude.json` and
/// prints the `oauthAccount.emailAddress` so the user can verify which Anthropic
/// account the Keychain entry currently holds a token for.
///
/// Usage: `cyolo profile whoami`
pub fn whoami(args: &[String]) -> Result<(), CyoloError> {
    if !args.is_empty() {
        eprintln!("{} cyolo profile whoami", "Usage:".yellow().bold());
        return Err(CyoloError::NonZeroExit(1));
    }

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

/// Read `oauthAccount.emailAddress` from `<config_dir>/.claude.json` and
/// return it when present.  Silently returns `None` if the file is missing,
/// unreadable, or does not contain the expected nested field — this is a
/// best-effort status read.
fn read_oauth_email(config_dir: &std::path::Path) -> Option<String> {
    let path = config_dir.join(".claude.json");
    let bytes = std::fs::read(&path).ok()?;
    let value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    value
        .get("oauthAccount")?
        .get("emailAddress")?
        .as_str()
        .map(str::to_owned)
}

/// Re-create shared symlinks for an already-registered profile.
///
/// Idempotent: existing correct symlinks are left as-is.
///
/// Usage: `cyolo profile link <name>`
pub fn link(args: &[String]) -> Result<(), CyoloError> {
    if args.len() != 1 {
        eprintln!("{} cyolo profile link <name>", "Usage:".yellow().bold());
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

    println!("Symlinks updated for profile '{}'", name.green());
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
        eprintln!("{} cyolo profile current", "Usage:".yellow().bold());
        return Err(CyoloError::NonZeroExit(1));
    }
    let resolved = crate::detect::resolve_profile()?;
    match resolved {
        Some(profile) => {
            if let Some(name) = &profile.name {
                println!("{} {}", "profile:".bold(), name.green());
            }
            println!("{} {}", "config_dir:".bold(), profile.config_dir.display().to_string().green());
            println!("{} {}", "source:".bold(), profile.source.green());
        }
        None => {
            println!("{}", "No profile detected. Using default Claude configuration (~/.claude).".dimmed());
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
                Some(name) => println!("Default profile: {}", name.green()),
                None => println!("{}", "No default profile set.".dimmed()),
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
                println!("Default profile set to: {}", name.green());
                Ok(())
            }
        }
        _ => {
            eprintln!("{} cyolo profile default [name | --unset]", "Usage:".yellow().bold());
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

/// What the interactive init menu resolved from the user's input line.
#[derive(Debug, PartialEq)]
pub(crate) enum MenuChoice {
    /// Zero-based index into the sorted profile list.
    Pick(usize),
    /// Register a fresh profile then bind to it.
    New,
    /// Do nothing, exit cleanly.
    Quit,
    /// Input did not match any option.
    Invalid,
}

/// Outcome of [`interactive_init_menu`] — distinguishes the "already launched
/// claude during the picker" case from the "just wrote a marker" case so a
/// caller that was about to run claude itself (bare `cyolo`) can avoid a
/// surprise double-launch.
#[derive(Debug, PartialEq)]
pub(crate) enum PickerOutcome {
    /// User picked an existing registered profile; `.claude-profile.json` was
    /// written. Caller should re-resolve and launch claude normally.
    MarkerWritten,
    /// User chose "new": a fresh profile was registered and `claude /login`
    /// was already launched (and exited) inside `add()`, and then the marker
    /// was written. Caller should **not** launch claude again.
    NewProfileRegistered,
    /// User quit without doing anything. No marker written.
    Quit,
}

/// Parse one line of user input from the interactive init menu.
///
/// Accepts:
///   * `<digit>`     — 1-based index; returned as 0-based `Pick`
///   * `n` / `new`   — `New`
///   * `q` / `quit`  — `Quit`
///   * empty line    — `Quit` (treat blank enter as "not now")
///   * anything else — `Invalid`
pub(crate) fn parse_menu_input(input: &str, profile_count: usize) -> MenuChoice {
    let s = input.trim().to_lowercase();
    if s.is_empty() || s == "q" || s == "quit" {
        return MenuChoice::Quit;
    }
    if s == "n" || s == "new" {
        return MenuChoice::New;
    }
    if let Ok(n) = s.parse::<usize>()
        && n >= 1
        && n <= profile_count
    {
        return MenuChoice::Pick(n - 1);
    }
    MenuChoice::Invalid
}

/// `true` when both stdin and stdout are connected to a terminal.
///
/// Gates interactive prompts: we never want to hang a CI run or a piped
/// invocation (`cyolo profile init | tee ...`) waiting for stdin.
pub(crate) fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Read a single trimmed line from stdin, returning an empty string on EOF.
fn read_line_trimmed() -> Result<String, CyoloError> {
    use std::io::BufRead as _;
    let stdin = std::io::stdin();
    let mut line = String::new();
    stdin
        .lock()
        .read_line(&mut line)
        .map_err(|e| CyoloError::ConfigIoError {
            context: "failed to read from stdin".into(),
            source: e,
        })?;
    Ok(line.trim().to_owned())
}

/// Write `.claude-profile.json` in the current working directory pointing at
/// `name`.  Fails when a file or symlink already exists at that path.
///
/// As a best-effort side effect, when the directory is inside a git repo we
/// append `.claude-profile.json` to `<gitdir>/info/exclude` so the marker
/// stays untracked without modifying the committed `.gitignore`. Failures
/// here are swallowed — marker creation succeeds either way.
fn write_profile_marker(name: &str) -> Result<(), CyoloError> {
    let cwd = std::env::current_dir().map_err(|e| CyoloError::ConfigIoError {
        context: "could not determine current directory".into(),
        source: e,
    })?;
    let profile_path = cwd.join(".claude-profile.json");

    // symlink_metadata catches broken symlinks (exists() returns false for them).
    if std::fs::symlink_metadata(&profile_path).is_ok() {
        eprintln!(
            "{} .claude-profile.json already exists in {}",
            "error:".red().bold(),
            cwd.display()
        );
        return Err(CyoloError::NonZeroExit(1));
    }

    let contents = serde_json::to_string_pretty(&serde_json::json!({"name": name}))
        .expect("JSON serialization of simple object cannot fail");
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&profile_path)
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to create {}", profile_path.display()),
            source: e,
        })?;
    file.write_all(format!("{contents}\n").as_bytes())
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to write {}", profile_path.display()),
            source: e,
        })?;

    println!(
        "Created {} (profile: {})",
        ".claude-profile.json".green(),
        name.green()
    );

    // Best-effort: mark the file as git-ignored via <gitdir>/info/exclude so
    // we do not require the user to edit the committed `.gitignore`.
    if let Some(gitdir) = crate::git::find_gitdir(&cwd)
        && let Ok(true) = crate::git::ensure_exclude_entry(&gitdir, ".claude-profile.json")
    {
        println!(
            "{} added {} to {}",
            "↳".dimmed(),
            ".claude-profile.json".dimmed(),
            format!("{}/info/exclude", gitdir.display()).dimmed()
        );
    }

    Ok(())
}

/// Show the interactive profile picker for `cyolo profile init` and the
/// auto-picker fired by bare `cyolo` in an unbound directory.
///
/// Caller must have confirmed we are on a TTY. Returns a [`PickerOutcome`]
/// that describes what (if anything) was done so the caller can decide
/// whether to fall through and launch `claude`. Real failures (registration,
/// marker write) still propagate as errors.
pub(crate) fn interactive_init_menu() -> Result<PickerOutcome, CyoloError> {
    let cfg = CyoloConfig::load()?;

    // Sorted profile names for stable indexing across invocations.
    let names: Vec<String> = cfg.profiles.keys().cloned().collect();

    let width = names.iter().map(String::len).max().unwrap_or(0);

    println!(
        "{} no profile is bound to this directory. Pick one:",
        "ℹ".cyan().bold()
    );
    println!();

    if names.is_empty() {
        println!("  {}", "(no profiles registered yet)".dimmed());
    } else {
        for (i, name) in names.iter().enumerate() {
            let profile = &cfg.profiles[name];
            let email = read_oauth_email(&profile.config_dir)
                .map(|e| e.green().to_string())
                .unwrap_or_else(|| "(needs login)".yellow().to_string());
            println!(
                "  {index}) {pad}  {email}",
                index = (i + 1).to_string().bold(),
                pad = format!("{name:<width$}").bold(),
            );
        }
    }
    println!("  {}) {}", "n".bold(), "new    register a new profile + /login");
    println!("  {}) {}", "q".bold(), "quit   do nothing");
    println!();

    use std::io::Write as _;
    print!("{} ", "Selection:".bold());
    std::io::stdout().flush().ok();

    let raw = read_line_trimmed()?;
    match parse_menu_input(&raw, names.len()) {
        MenuChoice::Pick(i) => {
            write_profile_marker(&names[i])?;
            Ok(PickerOutcome::MarkerWritten)
        }
        MenuChoice::New => {
            print!("{} ", "Name for new profile:".bold());
            std::io::stdout().flush().ok();
            let new_name = read_line_trimmed()?;
            if new_name.is_empty() {
                eprintln!("{} profile name cannot be empty", "error:".red().bold());
                return Err(CyoloError::NonZeroExit(1));
            }
            // `add` registers the profile *and* (unless `--no-login`) launches
            // claude for `/login`. The login session is a blocking, interactive
            // run of claude, so the caller of this picker must not double-launch.
            add(&[new_name.clone()])?;
            write_profile_marker(&new_name)?;
            Ok(PickerOutcome::NewProfileRegistered)
        }
        MenuChoice::Quit => {
            println!("{}", "No change. Run `cyolo profile init <name>` when ready.".dimmed());
            Ok(PickerOutcome::Quit)
        }
        MenuChoice::Invalid => {
            eprintln!(
                "{} unrecognized selection '{}'",
                "error:".red().bold(),
                raw.bold()
            );
            Err(CyoloError::NonZeroExit(1))
        }
    }
}

/// Create `.claude-profile.json` in the current working directory.
///
/// Resolution order:
///   1. Name given as argument
///   2. No args + default profile set → use default
///   3. No args + no default + TTY → interactive menu
///   4. No args + no default + non-TTY → error (unchanged, predictable for CI)
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
                if is_interactive() {
                    // `profile init` only cares whether the picker returns
                    // cleanly; the caller in cli.rs inspects the outcome.
                    interactive_init_menu()?;
                    return Ok(());
                }
                eprintln!(
                    "{} no profile name given and no default profile set",
                    "error:".red().bold()
                );
                eprintln!("{} cyolo profile init <name>", "Usage:".yellow().bold());
                return Err(CyoloError::NonZeroExit(1));
            }
        },
        1 => args[0].clone(),
        _ => {
            eprintln!("{} cyolo profile init <name>", "Usage:".yellow().bold());
            return Err(CyoloError::NonZeroExit(1));
        }
    };

    // Validate the name exists in config.profiles
    if !cfg.profiles.contains_key(&name) {
        return Err(CyoloError::ProfileNotFound { name });
    }

    write_profile_marker(&name)
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
    use std::sync::Once;

    // Disable ANSI color output once per test binary so eprintln!/println!
    // calls from tested code don't pollute captured output or break any
    // future assertions on stderr/stdout strings.
    static INIT_COLORS: Once = Once::new();
    fn setup() {
        INIT_COLORS.call_once(|| owo_colors::set_override(false));
    }

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_dispatch_unknown_subcommand_returns_error() {
        setup();
        let result = dispatch(&args(&["unknown"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_current_rejects_extra_args() {
        setup();
        let result = current(&args(&["unexpected"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatch_no_args_shows_help() {
        setup();
        let result = dispatch(&args(&[]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_dispatch_routes_to_default() {
        // "default" with 0 extra args calls profile_default(&[]), which
        // prints the current default (or "No default profile set.") and
        // returns Ok. The unknown-command catch-all returns Err, so Ok
        // proves the routing.
        setup();
        let result = dispatch(&args(&["default"]));
        assert!(result.is_ok());
    }

    #[test]
    fn test_dispatch_routes_to_init() {
        // "init" with an unregistered name returns ProfileNotFound (contains
        // "not found"), not the NonZeroExit(1) from the unknown-command
        // catch-all (which contains "unknown profile command").
        setup();
        let result = dispatch(&args(&["init", "__test_no_such_profile__"]));
        let err = result.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("not found"), "expected ProfileNotFound, got: {msg}");
    }

    #[test]
    fn test_profile_default_too_many_args() {
        setup();
        let result = profile_default(&args(&["a", "b"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_profile_init_too_many_args() {
        setup();
        let result = profile_init(&args(&["a", "b"]));
        assert!(result.is_err());
    }

    #[test]
    fn test_login_requires_one_arg() {
        setup();
        assert!(login(&args(&[])).is_err());
        assert!(login(&args(&["a", "b"])).is_err());
    }

    #[test]
    fn test_whoami_rejects_extra_args() {
        setup();
        assert!(whoami(&args(&["unexpected"])).is_err());
    }

    #[test]
    fn test_read_oauth_email_extracts_nested_field() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(".claude.json"),
            r#"{"oauthAccount":{"emailAddress":"test@example.com","accountUuid":"u"}}"#,
        )
        .unwrap();
        assert_eq!(
            read_oauth_email(dir.path()),
            Some("test@example.com".to_string())
        );
    }

    #[test]
    fn test_read_oauth_email_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(read_oauth_email(dir.path()), None);
    }

    #[test]
    fn test_read_oauth_email_missing_oauth_account_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".claude.json"), r#"{"userID":"abc"}"#).unwrap();
        assert_eq!(read_oauth_email(dir.path()), None);
    }

    #[test]
    fn test_read_oauth_email_invalid_json_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(".claude.json"), "not json").unwrap();
        assert_eq!(read_oauth_email(dir.path()), None);
    }

    #[test]
    fn test_add_rejects_missing_name() {
        setup();
        assert!(add(&args(&[])).is_err());
        // Flag-only (no positional) must still fail, proving --no-login is
        // filtered before the name lookup.
        assert!(add(&args(&["--no-login"])).is_err());
        assert!(add(&args(&["--no-share", "--no-login"])).is_err());
    }

    #[test]
    fn test_parse_menu_input_pick_valid_index() {
        assert_eq!(parse_menu_input("1", 3), MenuChoice::Pick(0));
        assert_eq!(parse_menu_input("3", 3), MenuChoice::Pick(2));
        assert_eq!(parse_menu_input("  2  ", 3), MenuChoice::Pick(1));
    }

    #[test]
    fn test_parse_menu_input_rejects_out_of_range() {
        assert_eq!(parse_menu_input("0", 3), MenuChoice::Invalid);
        assert_eq!(parse_menu_input("4", 3), MenuChoice::Invalid);
        // Empty list: every index must be Invalid.
        assert_eq!(parse_menu_input("1", 0), MenuChoice::Invalid);
    }

    #[test]
    fn test_parse_menu_input_new_aliases() {
        assert_eq!(parse_menu_input("n", 2), MenuChoice::New);
        assert_eq!(parse_menu_input("N", 2), MenuChoice::New);
        assert_eq!(parse_menu_input("new", 2), MenuChoice::New);
        assert_eq!(parse_menu_input("NEW", 2), MenuChoice::New);
    }

    #[test]
    fn test_parse_menu_input_quit_aliases_and_empty() {
        assert_eq!(parse_menu_input("q", 2), MenuChoice::Quit);
        assert_eq!(parse_menu_input("quit", 2), MenuChoice::Quit);
        assert_eq!(parse_menu_input("", 2), MenuChoice::Quit);
        assert_eq!(parse_menu_input("   ", 2), MenuChoice::Quit);
    }

    #[test]
    fn test_parse_menu_input_invalid_tokens() {
        assert_eq!(parse_menu_input("x", 2), MenuChoice::Invalid);
        assert_eq!(parse_menu_input("-1", 2), MenuChoice::Invalid);
        assert_eq!(parse_menu_input("1.5", 2), MenuChoice::Invalid);
    }
}
