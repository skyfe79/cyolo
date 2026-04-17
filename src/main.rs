mod cli;
mod config;
mod detect;
mod error;
mod profile;
mod runner;
mod symlink;

use error::CyoloError;

fn main() {
    if let Err(e) = cli::route() {
        match e {
            CyoloError::NonZeroExit(code) => std::process::exit(code),
            _ => {
                eprintln!("{e}");
                std::process::exit(1);
            }
        }
    }
}
