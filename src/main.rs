mod cli;
mod config;
mod error;
mod runner;

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
