use chrono::Duration;
use sea_orm::*;
use serenity::all::*;

use crate::database::{BotDatabase, entities};

pub type FlushInfo = entities::pending_flushes::Model;

pub struct FlushService;

impl FlushService {
    /// Check if a message has an associated flush
    pub async fn has_flush(message: &Message) -> Result<bool, DbErr> {
        let message_id = message.id.get() as i64;
        
        let count = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .count(BotDatabase::get().db())
            .await?;

        Ok(count > 0)
    }

    /// Add a new flush record
    pub async fn add_flush(
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
            .exec(BotDatabase::get().db())
            .await?;

        Ok(())
    }

    /// Get flush information by message ID
    pub async fn get_flush(message_id: MessageId) -> Result<Option<FlushInfo>, DbErr> {
        let message_id = message_id.get() as i64;
        
        let flush_info = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .one(BotDatabase::get().db())
            .await?;

        Ok(flush_info)
    }

    /// Remove a flush record by message ID
    pub async fn remove_flush(message_id: MessageId) -> Result<(), DbErr> {
        let message_id = message_id.get() as i64;
        
        entities::PendingFlushes::delete_many()
            .filter(
                entities::pending_flushes::Column::MessageId.eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id))
            )
            .exec(BotDatabase::get().db())
            .await?;
        Ok(())
    }

    /// Clean up old flush records
    pub async fn clean_old_flushes(dur: Duration) -> Result<(), DbErr> {
        let now = chrono::Utc::now();
        let bound = now - dur;

        entities::PendingFlushes::delete_many()
            .filter(entities::pending_flushes::Column::CreatedAt.lt(bound))
            .exec(BotDatabase::get().db())
            .await?;

        Ok(())
    }
}