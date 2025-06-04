use std::path::PathBuf;

use clap::Parser;

pub mod config;
pub mod database;
pub mod error;
pub mod framework;
pub mod handler;

#[derive(Parser)]
pub struct Args {
    #[clap(short, long, default_value = "config.json")]
    /// Path to the configuration file
    pub config: PathBuf,
    /// Path to the Redb database file
    #[clap(short, long, default_value = "redb.db")]
    pub redb: PathBuf,
}
