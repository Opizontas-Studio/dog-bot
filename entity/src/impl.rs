use sea_orm::sqlx::types::chrono::{DateTime, Utc};
use serenity::all::*;

use crate::messages::Model as Messages;
impl Messages {
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp.into()
    }
}

use crate::pending_flushes::Model as PendingFlushes;
impl PendingFlushes {
    pub fn message_id(&self) -> MessageId {
        MessageId::new(self.message_id as u64)
    }
    pub fn notification_id(&self) -> MessageId {
        MessageId::new(self.notification_id as u64)
    }
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::new(self.channel_id as u64)
    }
    pub fn toilet_id(&self) -> ChannelId {
        ChannelId::new(self.toilet_id as u64)
    }
    pub fn author_id(&self) -> UserId {
        UserId::new(self.author_id as u64)
    }
    pub fn flusher_id(&self) -> UserId {
        UserId::new(self.flusher_id as u64)
    }
    pub fn threshold(&self) -> u64 {
        self.threshold_count as u64
    }
    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at.into()
    }
}
