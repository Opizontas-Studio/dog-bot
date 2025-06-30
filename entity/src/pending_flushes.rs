use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
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
    pub fn toilet(&self) -> serenity::model::id::ChannelId {
        serenity::model::id::ChannelId::from(self.toilet_id as u64)
    }

    pub fn flusher(&self) -> serenity::model::id::UserId {
        serenity::model::id::UserId::from(self.flusher_id as u64)
    }

    pub fn message_id(&self) -> serenity::model::id::MessageId {
        serenity::model::id::MessageId::from(self.message_id as u64)
    }

    pub fn notification_id(&self) -> serenity::model::id::MessageId {
        serenity::model::id::MessageId::from(self.notification_id as u64)
    }

    pub fn channel_id(&self) -> serenity::model::id::ChannelId {
        serenity::model::id::ChannelId::from(self.channel_id as u64)
    }

    pub fn threshold(&self) -> u64 {
        self.threshold_count as u64
    }
}