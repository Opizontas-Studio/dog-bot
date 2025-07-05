use std::{path::Path, sync::LazyLock};

use clap::Parser;
use crossbeam::queue::ArrayQueue;
use sea_orm::{Database, DatabaseConnection, DbErr};

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
    message_insert_queue: ArrayQueue<entity::messages::ActiveModel>,
}

const DEFAULT_QUEUE_SIZE: usize = 100;

impl BotDatabase {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, DbErr> {
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let db = Database::connect(&database_url).await?;

        Ok(BotDatabase {
            db,
            message_insert_queue: ArrayQueue::new(DEFAULT_QUEUE_SIZE),
        })
    }

    pub async fn new_memory() -> Result<Self, DbErr> {
        let db = Database::connect("sqlite::memory:").await?;
        Ok(BotDatabase {
            db,
            message_insert_queue: ArrayQueue::new(DEFAULT_QUEUE_SIZE),
        })
    }

    pub fn inner(&self) -> &DatabaseConnection {
        &self.db
    }

    pub fn message_queue(&self) -> &ArrayQueue<entity::messages::ActiveModel> {
        &self.message_insert_queue
    }
}
