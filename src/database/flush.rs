use chrono::{Duration, NaiveDateTime};
use serde::{Deserialize, Serialize};
use serenity::all::*;
use sqlx::FromRow;

use crate::database::BotDatabase;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FlushInfo {
    pub message_id: i64,
    pub notification_id: i64,
    pub channel_id: i64,
    pub toilet_id: i64,
    pub author_id: i64,
    pub flusher_id: i64,
    pub threshold_count: i64,
    pub created_at: NaiveDateTime,
}

impl FlushInfo {
    pub fn toilet(&self) -> ChannelId {
        ChannelId::from(self.toilet_id as u64)
    }
    pub fn flusher(&self) -> UserId {
        UserId::from(self.flusher_id as u64)
    }
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id as u64)
    }
    pub fn notification_id(&self) -> MessageId {
        MessageId::from(self.notification_id as u64)
    }
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id as u64)
    }
    pub fn threshold(&self) -> u64 {
        self.threshold_count as u64
    }
}

impl BotDatabase {
    pub async fn has_flush(&self, message: &Message) -> Result<bool, sqlx::Error> {
        let message_id = message.id.get() as i64;
        let notification_id = message.id.get() as i64;
        let result = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM pending_flushes WHERE message_id = ? OR notification_id = ?",
            message_id,
            notification_id
        )
        .fetch_one(self.pool())
        .await?;

        Ok(result > 0)
    }

    pub async fn add_flush(
        &self,
        message: &Message,
        notify: &Message,
        flusher: UserId,
        toilet: ChannelId,
        threshold: u64,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();
        let message_id = message.id.get() as i64;
        let notification_id = notify.id.get() as i64;
        let channel_id = message.channel_id.get() as i64;
        let toilet_id = toilet.get() as i64;
        let author_id = message.author.id.get() as i64;
        let flusher_id = flusher.get() as i64;
        let threshold = threshold as i64;

        // Insert single record with both message_id and notification_id
        sqlx::query!(
            r#"--sql
            INSERT INTO pending_flushes 
            (message_id, notification_id, channel_id, toilet_id, author_id, flusher_id, threshold_count, created_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            message_id,
            notification_id,
            channel_id,
            toilet_id,
            author_id,
            flusher_id,
            threshold,
            now
        )
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_flush(&self, message_id: MessageId) -> Result<Option<FlushInfo>, sqlx::Error> {
        let message_id = message_id.get() as i64;
        let flush_info = sqlx::query_as!(
            FlushInfo,
            r#"--sql
            SELECT message_id, notification_id, channel_id, toilet_id, author_id, flusher_id, threshold_count, created_at
            FROM pending_flushes 
            WHERE message_id = ? OR notification_id = ?
            LIMIT 1
            "#,
            message_id,
            message_id
        )
        .fetch_optional(self.pool())
        .await?;

        Ok(flush_info)
    }

    pub async fn remove_flush(&self, message_id: MessageId) -> Result<(), sqlx::Error> {
        let message_id = message_id.get() as i64;
        // Remove flush by either message_id or notification_id
        sqlx::query!(
            "DELETE FROM pending_flushes WHERE message_id = ? OR notification_id = ?",
            message_id,
            message_id
        )
        .execute(self.pool())
        .await?;
        Ok(())
    }

    pub async fn clean_flushes(&self, dur: Duration) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();
        let bound = now - dur;

        sqlx::query!("DELETE FROM pending_flushes WHERE created_at < ?", bound)
            .execute(self.pool())
            .await?;

        Ok(())
    }
}
