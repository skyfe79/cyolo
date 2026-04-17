mod cli;
mod config;
mod detect;
mod diet;
mod error;
mod profile;
mod runner;
mod symlink;

use error::CyoloError;
use owo_colors::{set_override, OwoColorize};
use std::io::IsTerminal;

fn main() {
    if !std::io::stderr().is_terminal() {
        set_override(false);
    }

    if let Err(e) = cli::route() {
        match e {
            CyoloError::NonZeroExit(code) => std::process::exit(code),
            _ => {
                eprintln!("{} {e}", "error:".red().bold());
                std::process::exit(1);
            }
        }
    }
}
