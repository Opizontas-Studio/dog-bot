mod active;
mod channel;
mod codec;
mod flush;
use clap::Parser;
use redb::{Database, Error};
use std::{path::Path, sync::LazyLock};

use crate::Args;

pub static DB: LazyLock<BotDatabase> =
    LazyLock::new(|| BotDatabase::new(Args::parse().redb).expect("Failed to initialize database"));

pub struct BotDatabase {
    db: Database,
}

impl BotDatabase {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Box<Error>> {
        let db = Database::builder()
            .create_with_file_format_v3(true)
            .create(path)
            .map_err(Error::from)?;
        Ok(BotDatabase { db })
    }
}
