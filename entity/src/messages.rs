use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "messages")]
pub struct Model {
    #[sea_orm(primary_key, unique, auto_increment = false)]
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
    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }
}
