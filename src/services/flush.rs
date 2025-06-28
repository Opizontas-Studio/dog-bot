use chrono::Duration;
use sea_orm::*;
use serenity::all::*;

use crate::database::{BotDatabase, entities};

pub type FlushInfo = entities::pending_flushes::Model;

pub struct FlushService;

impl FlushService {
    /// Check if a message has an associated flush
    pub async fn has_flush(message: &Message) -> Result<bool, DbErr> {
        let message_id = message.id.get();

        let count = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId
                    .eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id)),
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
            message_id: Set(message.id.get()),
            notification_id: Set(notify.id.get()),
            channel_id: Set(message.channel_id.get()),
            toilet_id: Set(toilet.get()),
            author_id: Set(message.author.id.get()),
            flusher_id: Set(flusher.get()),
            threshold_count: Set(threshold),
            created_at: Set(chrono::Utc::now()),
        };

        entities::PendingFlushes::insert(flush)
            .exec(BotDatabase::get().db())
            .await?;

        Ok(())
    }

    /// Get flush information by message ID
    pub async fn get_flush(message_id: MessageId) -> Result<Option<FlushInfo>, DbErr> {
        let message_id = message_id.get();

        let flush_info = entities::PendingFlushes::find()
            .filter(
                entities::pending_flushes::Column::MessageId
                    .eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id)),
            )
            .one(BotDatabase::get().db())
            .await?;

        Ok(flush_info)
    }

    /// Remove a flush record by message ID
    pub async fn remove_flush(message_id: MessageId) -> Result<(), DbErr> {
        let message_id = message_id.get();

        entities::PendingFlushes::delete_many()
            .filter(
                entities::pending_flushes::Column::MessageId
                    .eq(message_id)
                    .or(entities::pending_flushes::Column::NotificationId.eq(message_id)),
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
