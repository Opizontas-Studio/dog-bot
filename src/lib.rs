use std::path::PathBuf;

use clap::Parser;

pub mod commands;
pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod services;
pub mod utils;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "config.json")]
    /// Path to the configuration file
    pub config: PathBuf,
    /// Path to the database file
    #[clap(short, long, default_value = "sqlite.db")]
    pub db: PathBuf,
}
