mod flush;
mod messages;
use clap::Parser;
use sqlx::SqlitePool;
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
    pool: SqlitePool,
}

impl BotDatabase {
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, sqlx::Error> {
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let pool = SqlitePool::connect(&database_url).await?;

        // Initialize tables
        Self::init_tables(&pool).await?;

        Ok(BotDatabase { pool })
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    async fn init_tables(pool: &SqlitePool) -> Result<(), sqlx::Error> {
        // Messages table - records all user messages
        sqlx::query!(
            r#"--sql
            CREATE TABLE IF NOT EXISTS messages (
                message_id INTEGER PRIMARY KEY NOT NULL,
                user_id INTEGER NOT NULL,
                guild_id INTEGER NOT NULL,
                channel_id INTEGER NOT NULL,
                timestamp DATETIME NOT NULL
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_messages_user_guild ON messages(user_id, guild_id)",
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_messages_guild_channel ON messages(guild_id, channel_id)"
        ).execute(pool).await?;

        sqlx::query!("CREATE INDEX IF NOT EXISTS idx_messages_timestamp ON messages(timestamp)")
            .execute(pool)
            .await?;

        // Flush system
        sqlx::query!(
            r#"--sql
            CREATE TABLE IF NOT EXISTS pending_flushes (
                message_id INTEGER PRIMARY KEY,
                notification_id INTEGER NOT NULL,
                channel_id INTEGER NOT NULL,
                toilet_id INTEGER NOT NULL,
                author_id INTEGER NOT NULL,
                flusher_id INTEGER NOT NULL,
                threshold_count INTEGER NOT NULL,
                created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_pending_flushes_notification ON pending_flushes(notification_id)"
        ).execute(pool).await?;

        sqlx::query!(
            "CREATE INDEX IF NOT EXISTS idx_pending_flushes_created_at ON pending_flushes(created_at)"
        ).execute(pool).await?;

        Ok(())
    }
}
