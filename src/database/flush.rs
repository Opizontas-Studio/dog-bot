use chrono::Duration;
use sea_orm::*;
use serenity::all::*;

use crate::database::{BotDatabase, entities};

pub type FlushInfo = entities::pending_flushes::Model;

impl BotDatabase {
    pub async fn has_flush(&self, message: &Message) -> Result<bool, DbErr> {
        let message_id = message.id.get() as i64;
        
        let count = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .count(self.db())
            .await?;

        Ok(count > 0)
    }

    pub async fn add_flush(
        &self,
        message: &Message,
        notify: &Message,
        flusher: UserId,
        toilet: ChannelId,
        threshold: u64,
    ) -> Result<(), DbErr> {
        let flush = entities::pending_flushes::ActiveModel {
            message_id: Set(message.id.get() as i64),
            notification_id: Set(notify.id.get() as i64),
            channel_id: Set(message.channel_id.get() as i64),
            toilet_id: Set(toilet.get() as i64),
            author_id: Set(message.author.id.get() as i64),
            flusher_id: Set(flusher.get() as i64),
            threshold_count: Set(threshold as i64),
            created_at: Set(chrono::Utc::now()),
        };

        entities::PendingFlushes::insert(flush)
            .exec(self.db())
            .await?;

        Ok(())
    }

    pub async fn get_flush(&self, message_id: MessageId) -> Result<Option<FlushInfo>, DbErr> {
        let message_id = message_id.get() as i64;
        
        let flush_info = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .one(self.db())
            .await?;

        Ok(flush_info)
    }

    pub async fn remove_flush(&self, message_id: MessageId) -> Result<(), DbErr> {
        let message_id = message_id.get() as i64;
        
        entities::PendingFlushes::delete_many()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .exec(self.db())
            .await?;
        Ok(())
    }

    pub async fn clean_flushes(&self, dur: Duration) -> Result<(), DbErr> {
        let now = chrono::Utc::now();
        let bound = now - dur;

        entities::PendingFlushes::delete_many()
            .filter(entities::pending_flushes::Column::CreatedAt.lt(bound))
            .exec(self.db())
            .await?;

        Ok(())
    }
}
