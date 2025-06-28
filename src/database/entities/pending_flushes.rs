use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serenity::all::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "pending_flushes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: i64,
    pub notification_id: i64,
    pub channel_id: i64,
    pub toilet_id: i64,
    pub author_id: i64,
    pub flusher_id: i64,
    pub threshold_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
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
