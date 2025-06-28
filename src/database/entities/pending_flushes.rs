use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::all::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "pending_flushes")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: u64,
    pub notification_id: u64,
    pub channel_id: u64,
    pub toilet_id: u64,
    pub author_id: u64,
    pub flusher_id: u64,
    pub threshold_count: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn toilet(&self) -> ChannelId {
        ChannelId::from(self.toilet_id)
    }

    pub fn flusher(&self) -> UserId {
        UserId::from(self.flusher_id)
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

    pub fn threshold(&self) -> u64 {
        self.threshold_count
    }
}
