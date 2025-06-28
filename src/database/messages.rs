use chrono::{DateTime, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::*;
use serenity::all::*;

use crate::database::{BotDatabase, entities};

pub type MessageRecord = entities::messages::Model;

impl BotDatabase {
    pub fn messages(&self) -> Messages {
        Messages(self)
    }
}

pub struct Messages<'a>(&'a BotDatabase);

impl<'a> Messages<'a> {
    /// Record a message event
    pub async fn record(
        &self,
        message_id: MessageId,
        user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        timestamp: Timestamp,
    ) -> Result<(), DbErr> {
        let message = entities::messages::ActiveModel {
            message_id: Set(message_id.get() as i64),
            user_id: Set(user_id.get() as i64),
            guild_id: Set(guild_id.get() as i64),
            channel_id: Set(channel_id.get() as i64),
            timestamp: Set(timestamp.to_utc()),
        };

        entities::Messages::insert(message)
            .on_conflict(
                OnConflict::column(entities::messages::Column::MessageId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(self.0.db())
            .await?;
        Ok(())
    }

    /// Get user activity data for a specific guild
    pub async fn get_user_activity(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, DbErr> {
        let messages = entities::Messages::find()
            .filter(
                entities::messages::Column::UserId
                    .eq(user_id.get() as i64)
                    .and(entities::messages::Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_asc(entities::messages::Column::Timestamp)
            .all(self.0.db())
            .await?;

        Ok(messages.into_iter().map(|m| m.timestamp).collect())
    }

    /// Get channel statistics for a guild
    pub async fn get_channel_stats(
        &self,
        guild_id: GuildId,
    ) -> Result<Vec<(ChannelId, u64)>, DbErr> {
        use sea_orm::FromQueryResult;
        use sea_orm::sea_query::{Expr, Func, Order, Query};

        #[derive(FromQueryResult)]
        struct ChannelCount {
            channel_id: i64,
            message_count: i64,
        }

        const MESSAGE_COUNT: &str = "message_count";
        let query = Query::select()
            .column(entities::messages::Column::ChannelId)
            .expr_as(
                Func::count(Expr::col(entities::messages::Column::MessageId)),
                MESSAGE_COUNT,
            )
            .from(entities::messages::Entity)
            .and_where(entities::messages::Column::GuildId.eq(guild_id.get() as i64))
            .group_by_col(entities::messages::Column::ChannelId)
            .order_by(MESSAGE_COUNT, Order::Desc)
            .to_owned();

        let builder = self.0.db().get_database_backend();
        let statement = builder.build(&query);

        let results = ChannelCount::find_by_statement(statement)
            .all(self.0.db())
            .await?;

        Ok(results
            .into_iter()
            .map(|row| {
                (
                    ChannelId::new(row.channel_id as u64),
                    row.message_count as u64,
                )
            })
            .collect())
    }

    /// Get message records for a specific user in a guild
    pub async fn get_user_messages(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<MessageRecord>, DbErr> {
        let messages = entities::Messages::find()
            .filter(
                entities::messages::Column::UserId
                    .eq(user_id.get() as i64)
                    .and(entities::messages::Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_desc(entities::messages::Column::Timestamp)
            .all(self.0.db())
            .await?;

        Ok(messages)
    }

    /// Clear all message data (dangerous operation)
    pub async fn nuke(&self) -> Result<(), DbErr> {
        entities::Messages::delete_many().exec(self.0.db()).await?;
        Ok(())
    }
}
