mod active;
mod channel;
mod flush;
use chrono::{DateTime, Utc};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serenity::all::{MessageId, UserId};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supervisor {
    pub id: UserId,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub polls: Vec<MessageId>,
    #[serde(default)]
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PollStage {
    Proposal,
    Outdated,
    Polling,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: MessageId,
    pub proposer: UserId,
    pub stage: PollStage,
    pub signs_needed: u64,
    pub approves_needed: u64,
    pub approve_ratio_needed: f64,
    pub signatures: Vec<UserId>,
    pub approves: Vec<UserId>,
    pub rejects: Vec<UserId>,
}

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
        // Active user tracking
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS active_data (
                user_id INTEGER NOT NULL,
                guild_id INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                PRIMARY KEY (user_id, guild_id, timestamp)
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_active_data_user_guild ON active_data(user_id, guild_id)"
        ).execute(pool).await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_active_data_timestamp ON active_data(timestamp)",
        )
        .execute(pool)
        .await?;

        // Channel statistics
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS channel_data (
                guild_id INTEGER NOT NULL,
                channel_id INTEGER NOT NULL,
                message_count INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (guild_id, channel_id)
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_channel_data_guild ON channel_data(guild_id)")
            .execute(pool)
            .await?;

        // Flush system
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS pending_flushes (
                message_id INTEGER PRIMARY KEY,
                notification_id INTEGER NOT NULL,
                channel_id INTEGER NOT NULL,
                toilet_id INTEGER NOT NULL,
                author_id INTEGER NOT NULL,
                flusher_id INTEGER NOT NULL,
                threshold_count INTEGER NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
            )
            "#,
        )
        .execute(pool)
        .await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_pending_flushes_notification ON pending_flushes(notification_id)"
        ).execute(pool).await?;

        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_pending_flushes_created_at ON pending_flushes(created_at)"
        ).execute(pool).await?;

        Ok(())
    }
}
