use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use serenity::all::*;
use sqlx::FromRow;

use crate::database::BotDatabase;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MessageRecord {
    pub message_id: i64,
    pub user_id: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub timestamp: NaiveDateTime,
}

impl MessageRecord {
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId::from(self.user_id as u64)
    }

    pub fn guild_id(&self) -> GuildId {
        GuildId::from(self.guild_id as u64)
    }

    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id as u64)
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp.and_utc()
    }
}

impl BotDatabase {
    pub fn messages(&self) -> Messages {
        Messages(self)
    }
}

pub struct Messages<'a>(&'a BotDatabase);

impl<'a> Messages<'a> {
    /// Record a message event
    pub async fn record(
        &self,
        message_id: MessageId,
        user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        timestamp: Timestamp,
    ) -> Result<(), sqlx::Error> {
        let message_id = message_id.get() as i64;
        let user_id = user_id.get() as i64;
        let guild_id = guild_id.get() as i64;
        let channel_id = channel_id.get() as i64;
        let timestamp = timestamp.to_utc();
        sqlx::query!(
            "INSERT OR IGNORE INTO messages (message_id, user_id, guild_id, channel_id, timestamp) VALUES (?, ?, ?, ?, ?)",
            message_id,
            user_id,
            guild_id,
            channel_id,
            timestamp
        )
        .execute(self.0.pool())
        .await?;
        Ok(())
    }

    /// Get user activity data for a specific guild
    pub async fn get_user_activity(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, sqlx::Error> {
        let user_id_i = user_id.get() as i64;
        let guild_id = guild_id.get() as i64;

        let timestamps = sqlx::query_scalar!(
            "SELECT timestamp FROM messages WHERE user_id = ? AND guild_id = ? ORDER BY timestamp",
            user_id_i,
            guild_id
        )
        .fetch_all(self.0.pool())
        .await?;

        let data: Vec<DateTime<Utc>> = timestamps.into_iter().map(|ts| ts.and_utc()).collect();
        Ok(data)
    }

    /// Get channel statistics for a guild
    pub async fn get_channel_stats(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<(ChannelId, u64)>, sqlx::Error> {
        let guild_id = guild_id.get() as i64;
        let rows = sqlx::query!(
            r#"
            SELECT channel_id, COUNT(*) as "message_count!: i64"
            FROM messages
            WHERE guild_id = ?
            GROUP BY channel_id
            ORDER BY "message_count" DESC
            "#,
            guild_id
        )
        .fetch_all(self.0.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    ChannelId::new(row.channel_id as u64),
                    row.message_count as u64,
                )
            })
            .collect())
    }

    /// Get message records for a specific user in a guild
    pub async fn get_user_messages(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<MessageRecord>, sqlx::Error> {
        let user_id = user_id.get() as i64;
        let guild_id = guild_id.get() as i64;

        let messages = sqlx::query_as!(
            MessageRecord,
            r#"SELECT message_id , user_id, guild_id, channel_id, timestamp FROM messages WHERE user_id = ? AND guild_id = ? ORDER BY timestamp DESC"#,
            user_id,
            guild_id
        )
        .fetch_all(self.0.pool())
        .await?;

        Ok(messages)
    }

    /// Clear all message data (dangerous operation)
    pub async fn nuke(&self) -> Result<(), sqlx::Error> {
        sqlx::query!("DELETE FROM messages")
            .execute(self.0.pool())
            .await?;
        Ok(())
    }
}
