use chrono::Duration;
use serde::{Deserialize, Serialize};
use serenity::all::*;
use sqlx::Row;

use crate::database::BotDatabase;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlushInfo {
    pub message_id: u64,
    pub notification_id: u64,
    pub channel_id: u64,
    pub toilet: u64,
    pub author: u64,
    pub flusher: u64,
    pub threshold: u64,
}

impl FlushInfo {
    pub fn toilet(&self) -> ChannelId {
        ChannelId::from(self.toilet)
    }
    pub fn flusher(&self) -> UserId {
        UserId::from(self.flusher)
    }
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id)
    }
    pub fn notification_id(&self) -> MessageId {
        MessageId::from(self.notification_id)
    }
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id)
    }
}

impl BotDatabase {
    pub async fn has_flush(&self, message: &Message) -> Result<bool, sqlx::Error> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM pending_flushes WHERE message_id = ?")
            .bind(message.id.get() as i64)
            .fetch_one(self.pool())
            .await?;

        let count: i64 = row.get("count");
        Ok(count > 0)
    }

    pub async fn add_flush(
        &self,
        message: &Message,
        notify: &Message,
        flusher: UserId,
        toilet: ChannelId,
        threshold: u64,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now().timestamp();

        // Insert both message and notification entries
        sqlx::query(
            r#"
            INSERT INTO pending_flushes 
            (message_id, notification_id, channel_id, toilet_id, author_id, flusher_id, threshold_count, created_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(message.id.get() as i64)
        .bind(notify.id.get() as i64)
        .bind(message.channel_id.get() as i64)
        .bind(toilet.get() as i64)
        .bind(message.author.id.get() as i64)
        .bind(flusher.get() as i64)
        .bind(threshold as i64)
        .bind(now)
        .execute(self.pool())
        .await?;

        // Also insert notification entry for easy lookup
        sqlx::query(
            r#"
            INSERT INTO pending_flushes 
            (message_id, notification_id, channel_id, toilet_id, author_id, flusher_id, threshold_count, created_at) 
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(notify.id.get() as i64)
        .bind(notify.id.get() as i64)
        .bind(message.channel_id.get() as i64)
        .bind(toilet.get() as i64)
        .bind(message.author.id.get() as i64)
        .bind(flusher.get() as i64)
        .bind(threshold as i64)
        .bind(now)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    pub async fn get_flush(&self, message_id: MessageId) -> Result<Option<FlushInfo>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT message_id, notification_id, channel_id, toilet_id, author_id, flusher_id, threshold_count 
            FROM pending_flushes 
            WHERE message_id = ? 
            LIMIT 1
            "#
        )
        .bind(message_id.get() as i64)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            Ok(Some(FlushInfo {
                message_id: row.get::<i64, _>("message_id") as u64,
                notification_id: row.get::<i64, _>("notification_id") as u64,
                channel_id: row.get::<i64, _>("channel_id") as u64,
                toilet: row.get::<i64, _>("toilet_id") as u64,
                author: row.get::<i64, _>("author_id") as u64,
                flusher: row.get::<i64, _>("flusher_id") as u64,
                threshold: row.get::<i64, _>("threshold_count") as u64,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn remove_flush(&self, message_id: MessageId) -> Result<(), sqlx::Error> {
        // Get the flush info first to find related entries
        if let Some(info) = self.get_flush(message_id).await? {
            // Remove both message and notification entries
            sqlx::query("DELETE FROM pending_flushes WHERE message_id = ? OR message_id = ?")
                .bind(info.message_id as i64)
                .bind(info.notification_id as i64)
                .execute(self.pool())
                .await?;
        }
        Ok(())
    }

    pub async fn clean_flushes(&self, dur: Duration) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();
        let bound = (now - dur).timestamp();

        sqlx::query("DELETE FROM pending_flushes WHERE created_at < ?")
            .bind(bound)
            .execute(self.pool())
            .await?;

        Ok(())
    }
}
