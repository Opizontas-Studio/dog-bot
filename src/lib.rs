use std::path::PathBuf;

use clap::Parser;

pub mod config;
pub mod error;
pub mod handler;
pub mod framework;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "config.json")]
    /// Path to the configuration file
    pub config_path: PathBuf,
}
