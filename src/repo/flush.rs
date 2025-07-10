use chrono::Duration;
use entities::pending_flushes::*;
use sea_orm::{DbErr, Set, prelude::*};
use serenity::all::*;

use crate::{database::BotDatabase, error::BotError};

pub type FlushInfo = Model;
pub struct FlushRepo<'a>(&'a BotDatabase);
impl BotDatabase {
    /// Get a reference to the database
    pub fn flush(&self) -> FlushRepo<'_> {
        FlushRepo(self)
    }
}

impl FlushRepo<'_> {
    /// Check if a message has an associated flush
    pub async fn has(self, message: &Message) -> Result<bool, BotError> {
        self.get(message.id).await.map(|info| info.is_some())
    }

    /// Add a new flush record
    pub async fn insert(
        self,
        message: &Message,
        notify: &Message,
        flusher: UserId,
        toilet: ChannelId,
        threshold: u64,
        reason: Option<String>,
    ) -> Result<(), DbErr> {
        let flush = ActiveModel {
            message_id: Set(message.id.get() as i64),
            notification_id: Set(notify.id.get() as i64),
            channel_id: Set(message.channel_id.get() as i64),
            toilet_id: Set(toilet.get() as i64),
            author_id: Set(message.author.id.get() as i64),
            flusher_id: Set(flusher.get() as i64),
            threshold_count: Set(threshold as i64),
            created_at: Set(chrono::Utc::now().into()),
            reason: Set(reason),
        };

        flush.insert(self.0.inner()).await?;

        Ok(())
    }

    /// Get flush information by message ID
    pub async fn get(self, message_id: MessageId) -> Result<Option<FlushInfo>, BotError> {
        let message_id = message_id.get() as i64;

        Ok(Entity::find()
            .filter(
                Column::MessageId
                    .eq(message_id)
                    .or(Column::NotificationId.eq(message_id)),
            )
            .one(self.0.inner())
            .await?)
    }

    /// Remove a flush record by message ID
    pub async fn remove(self, message_id: MessageId) -> Result<(), BotError> {
        let message_id = message_id.get() as i64;

        Entity::delete_many()
            .filter(
                Column::MessageId
                    .eq(message_id)
                    .or(Column::NotificationId.eq(message_id)),
            )
            .exec(self.0.inner())
            .await?;
        Ok(())
    }

    /// Clean up old flush records
    pub async fn clean(self, dur: Duration) -> Result<(), BotError> {
        let now = chrono::Utc::now();
        let bound = now - dur;

        Entity::delete_many()
            .filter(Column::CreatedAt.lt(bound))
            .exec(self.0.inner())
            .await?;

        Ok(())
    }
}
