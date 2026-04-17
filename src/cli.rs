use crate::detect;
use crate::diet;
use crate::error::CyoloError;
use crate::profile;
use crate::runner;

/// Top-level command classification.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// `cyolo update` → `claude update`
    Update,
    /// `cyolo profile <...>` → profile management (stub in v1)
    Profile(Vec<String>),
    /// `cyolo diet <...>` → config cleanup (stub in v1)
    Diet(Vec<String>),
    /// Everything else → `claude --dangerously-skip-permissions <args>`
    Claude(Vec<String>),
}

/// Classify raw CLI arguments into a Command.
pub fn classify(args: &[String]) -> Command {
    match args.first().map(|s| s.as_str()) {
        Some("update") => Command::Update,
        Some("profile") => Command::Profile(args[1..].to_vec()),
        Some("diet") => Command::Diet(args[1..].to_vec()),
        _ => Command::Claude(args.to_vec()),
    }
}

/// Route execution based on CLI arguments.
pub fn route() -> Result<(), CyoloError> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match classify(&args) {
        Command::Update => runner::run_update(),
        Command::Profile(args) => profile::dispatch(&args),
        Command::Diet(args) => diet::dispatch(&args),
        Command::Claude(args) => {
            let resolved = detect::resolve_profile()?;
            let config_dir = resolved.as_ref().map(|r| r.config_dir.as_path());
            runner::run_claude(&args, config_dir)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn test_classify_update() {
        assert_eq!(classify(&args(&["update"])), Command::Update);
    }

    #[test]
    fn test_classify_profile() {
        assert_eq!(
            classify(&args(&["profile", "list"])),
            Command::Profile(args(&["list"]))
        );
    }

    #[test]
    fn test_classify_diet() {
        assert_eq!(
            classify(&args(&["diet", "--apply"])),
            Command::Diet(args(&["--apply"]))
        );
    }

    #[test]
    fn test_classify_passthrough_with_args() {
        assert_eq!(
            classify(&args(&["-p", "hello world"])),
            Command::Claude(args(&["-p", "hello world"]))
        );
    }

    #[test]
    fn test_classify_no_args() {
        assert_eq!(classify(&args(&[])), Command::Claude(vec![]));
    }

    #[test]
    fn test_classify_help_flag() {
        assert_eq!(
            classify(&args(&["--help"])),
            Command::Claude(args(&["--help"]))
        );
    }
}
