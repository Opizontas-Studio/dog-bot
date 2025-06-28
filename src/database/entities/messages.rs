use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serenity::all::*;

#[derive(Clone, Debug, DeriveEntityModel)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub message_id: u64,
    pub user_id: u64,
    pub guild_id: u64,
    pub channel_id: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id)
    }

    pub fn user_id(&self) -> UserId {
        UserId::from(self.user_id)
    }

    pub fn guild_id(&self) -> GuildId {
        GuildId::from(self.guild_id)
    }

    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id)
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}
