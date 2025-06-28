use chrono::{DateTime, Utc};
use sea_orm::sea_query::{OnConflict, SimpleExpr};
use sea_orm::*;
use serenity::all::*;

use crate::database::{
    BotDatabase,
    entities::{Messages, messages::*},
};

pub type MessageRecord = Model;

pub struct MessageService;

impl MessageService {
    /// Record a message event
    pub async fn record_message(
        message_id: MessageId,
        user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        timestamp: Timestamp,
    ) -> Result<(), DbErr> {
        let message = ActiveModel {
            message_id: Set(message_id.get() as i64),
            user_id: Set(user_id.get() as i64),
            guild_id: Set(guild_id.get() as i64),
            channel_id: Set(channel_id.get() as i64),
            timestamp: Set(timestamp.to_utc()),
        };

        Messages::insert(message)
            .on_conflict(
                OnConflict::column(Column::MessageId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(BotDatabase::get().db())
            .await?;
        Ok(())
    }

    /// Get user activity data for a specific guild
    pub async fn get_user_activity(
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, DbErr> {
        Messages::find()
            .select_only()
            .column(Column::Timestamp)
            .filter(
                Column::UserId
                    .eq(user_id.get() as i64)
                    .and(Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_asc(Column::Timestamp)
            .into_tuple()
            .all(BotDatabase::get().db())
            .await
    }

    /// Get channel statistics for a guild
    pub async fn get_channel_stats(
        guild_id: GuildId,
        top_n: usize,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<(ChannelId, u64)>, DbErr> {
        use sea_orm::sea_query::{Alias, Expr};

        const ALIAS: &str = "message_count";
        Ok(Messages::find()
            .select_only()
            .column(Column::ChannelId)
            .filter(Column::GuildId.eq(guild_id.get() as i64))
            .filter(from.map_or(SimpleExpr::Value(true.into()), |f| Column::Timestamp.gte(f)))
            .filter(to.map_or(SimpleExpr::Value(true.into()), |t| Column::Timestamp.lt(t)))
            .column_as(Column::MessageId.count(), ALIAS)
            .group_by(Column::ChannelId)
            .order_by_desc(Expr::col(Alias::new(ALIAS)))
            .limit(top_n as u64)
            .into_tuple::<(i64, i64)>()
            .all(BotDatabase::get().db())
            .await?
            .into_iter()
            .map(|(channel_id, count)| (ChannelId::new(channel_id as u64), count as u64))
            .collect())
    }

    /// Get user statistics for a guild
    pub async fn get_user_stats(
        guild_id: GuildId,
        top_n: usize,
        from: Option<DateTime<Utc>>,
        to: Option<DateTime<Utc>>,
    ) -> Result<Vec<(UserId, u64)>, DbErr> {
        use sea_orm::sea_query::{Alias, Expr};

        const ALIAS: &str = "message_count";
        Ok(Messages::find()
            .select_only()
            .column(Column::UserId)
            .filter(Column::GuildId.eq(guild_id.get() as i64))
            .filter(from.map_or(SimpleExpr::Value(true.into()), |f| Column::Timestamp.gte(f)))
            .filter(to.map_or(SimpleExpr::Value(true.into()), |t| Column::Timestamp.lt(t)))
            .column_as(Column::MessageId.count(), ALIAS)
            .group_by(Column::UserId)
            .order_by_desc(Expr::col(Alias::new(ALIAS)))
            .limit(top_n as u64)
            .into_tuple::<(i64, i64)>()
            .all(BotDatabase::get().db())
            .await?
            .into_iter()
            .map(|(user_id, count)| (UserId::new(user_id as u64), count as u64))
            .collect())
    }

    /// Get message records for a specific user in a guild
    pub async fn get_user_messages(
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<MessageRecord>, DbErr> {
        Ok(Messages::find()
            .filter(
                Column::UserId
                    .eq(user_id.get() as i64)
                    .and(Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_desc(Column::Timestamp)
            .all(BotDatabase::get().db())
            .await?)
    }

    /// Clear all message data (dangerous operation)
    pub async fn nuke_all_messages() -> Result<(), DbErr> {
        Messages::delete_many()
            .exec(BotDatabase::get().db())
            .await?;
        Ok(())
    }
}
