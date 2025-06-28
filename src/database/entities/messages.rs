use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serenity::all::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: i64,
    pub user_id: i64,
    pub guild_id: i64,
    pub channel_id: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id as u64)
    }

    pub fn user_id(&self) -> UserId {
        UserId::from(self.user_id as u64)
    }

    pub fn guild_id(&self) -> GuildId {
        GuildId::from(self.guild_id as u64)
    }

    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id as u64)
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}