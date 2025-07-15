use std::path::Path;

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, Statement};
use serenity::{all::Context, prelude::TypeMapKey};
use snafu::OptionExt;

use crate::error::BotError;

#[derive(Debug, Clone)]
pub struct BotDatabase {
    db: DatabaseConnection,
}

impl TypeMapKey for BotDatabase {
    type Value = BotDatabase;
}

pub(crate) trait GetDb {
    async fn db(&self) -> Result<BotDatabase, BotError>;
}

impl GetDb for Context {
    async fn db(&self) -> Result<BotDatabase, BotError> {
        self.data
            .read()
            .await
            .get::<BotDatabase>()
            .cloned()
            .whatever_context::<&str, BotError>("Failed to get BotDatabase from context")
    }
}

impl BotDatabase {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, BotError> {
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let db = Database::connect(&database_url).await?;

        Ok(BotDatabase { db })
    }

    pub async fn new_memory() -> Result<Self, BotError> {
        let db = Database::connect("sqlite::memory:").await?;
        Ok(BotDatabase { db })
    }

    pub fn inner(&self) -> &DatabaseConnection {
        &self.db
    }

    pub async fn size(&self) -> Result<i64, BotError> {
        let stmt = Statement::from_string(
            DbBackend::Sqlite,
            "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()",
        );

        let result = self.db.query_one(stmt).await?;
        if let Some(row) = result {
            let size: i64 = row.try_get("", "size")?;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    pub async fn vacuum(&self) -> Result<(), BotError> {
        let stmt = Statement::from_string(DbBackend::Sqlite, "VACUUM");
        self.db.execute(stmt).await?;
        Ok(())
    }
}
