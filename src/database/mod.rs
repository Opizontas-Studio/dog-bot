use clap::Parser;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::{path::Path, sync::LazyLock};

use crate::Args;

pub static DB: LazyLock<BotDatabase> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            BotDatabase::new(Args::parse().db)
                .await
                .expect("Failed to initialize database")
        })
    })
});

pub struct BotDatabase {
    db: DatabaseConnection,
}

impl BotDatabase {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, DbErr> {
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let db = Database::connect(&database_url).await?;

        Ok(BotDatabase { db })
    }

    pub async fn new_memory() -> Result<Self, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        Ok(BotDatabase { db })
    }

    pub fn inner(&self) -> &DatabaseConnection {
        &self.db
    }
}
