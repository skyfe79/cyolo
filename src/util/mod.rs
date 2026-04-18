//! Tiny cross-module helpers that have no natural home in any single
//! command.  Keep this module intentionally small — if something grows
//! past a few lines or starts acquiring dependencies it probably belongs
//! in its own module (see `mcp`, `git`, `symlink`).
//!
//! The functions here are pulled out specifically so that neither
//! `detect` nor any of the `commands::*` modules need to depend on each
//! other's internals.

use std::io::BufRead;
use std::path::PathBuf;

use crate::error::CyoloError;

/// Expand a leading `~` or `~/` to the user's home directory.
///
/// Returns the input unchanged when the path does not begin with a tilde
/// or when the home directory cannot be determined (unusual but possible
/// inside sandboxed CI).
pub fn expand_tilde(path: &str) -> PathBuf {
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

/// `true` when both stdin and stdout are connected to a terminal.
///
/// Gates interactive prompts: we never want to hang a CI run or a piped
/// invocation (`cyolo profile init | tee ...`) waiting for stdin.
pub fn is_interactive() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stdout().is_terminal()
}

/// Read a single trimmed line from stdin, returning an empty string on EOF.
pub fn read_line_trimmed() -> Result<String, CyoloError> {
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

#[cfg(test)]
mod tests;
