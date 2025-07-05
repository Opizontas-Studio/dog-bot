use chrono::{DateTime, FixedOffset, Utc};
use entity::messages::*;
use sea_orm::{DbErr, QueryOrder, QuerySelect, Set, prelude::*, sea_query::*};
use serenity::all::*;

use crate::database::BotDatabase;

pub type MessageRecord = Model;

pub(crate) trait MessageService {
    /// Record a message event
    async fn record(
        &self,
        message_id: MessageId,
        user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        timestamp: Timestamp,
    ) -> Result<(), DbErr>;

    /// Get user activity data for a specific guild
    async fn get_user_activity(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, DbErr>;

    /// Get channel statistics for a guild
    async fn get_channel_stats(
        &self,
        guild_id: GuildId,
        from: Option<impl Into<DateTime<FixedOffset>>>,
        to: Option<impl Into<DateTime<FixedOffset>>>,
    ) -> Result<Vec<(ChannelId, u64)>, DbErr>;

    /// Get user statistics for a guild
    async fn get_user_stats(
        &self,
        guild_id: GuildId,
        channel_ids: Option<&[ChannelId]>,
        from: Option<impl Into<DateTime<FixedOffset>>>,
        to: Option<impl Into<DateTime<FixedOffset>>>,
    ) -> Result<Vec<(UserId, u64)>, DbErr>;

    /// Get message records for a specific user in a guild
    #[allow(dead_code)]
    async fn get_user_messages(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<MessageRecord>, DbErr>;

    /// Clear all message data (dangerous operation)
    #[allow(dead_code)]
    async fn nuke(&self) -> Result<(), DbErr>;
}

pub struct DbMessage<'a>(&'a BotDatabase);
impl BotDatabase {
    /// Get a reference to the database
    pub fn message(&self) -> DbMessage<'_> {
        DbMessage(self)
    }
}

impl MessageService for DbMessage<'_> {
    /// Record a message event
    async fn record(
        &self,
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
            timestamp: Set(timestamp.to_utc().into()),
        };
        Entity::insert(message)
            .on_conflict(
                OnConflict::column(Column::MessageId)
                    .do_nothing()
                    .to_owned(),
            )
            .exec(self.0.inner())
            .await?;
        Ok(())
    }

    /// Get user activity data for a specific guild
    async fn get_user_activity(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, DbErr> {
        Entity::find()
            .select_only()
            .column(Column::Timestamp)
            .filter(
                Column::UserId
                    .eq(user_id.get() as i64)
                    .and(Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_asc(Column::Timestamp)
            .into_tuple()
            .all(self.0.inner())
            .await
    }

    /// Get channel statistics for a guild
    async fn get_channel_stats(
        &self,
        guild_id: GuildId,
        from: Option<impl Into<DateTime<FixedOffset>>>,
        to: Option<impl Into<DateTime<FixedOffset>>>,
    ) -> Result<Vec<(ChannelId, u64)>, DbErr> {
        use sea_orm::sea_query::{Alias, Expr};

        const ALIAS: &str = "message_count";
        Ok(Entity::find()
            .select_only()
            .column(Column::ChannelId)
            .filter(Column::GuildId.eq(guild_id.get() as i64))
            .filter(from.map_or(SimpleExpr::Value(true.into()), |f| {
                Column::Timestamp.gte(f.into())
            }))
            .filter(to.map_or(SimpleExpr::Value(true.into()), |t| {
                Column::Timestamp.lt(t.into())
            }))
            .column_as(Column::MessageId.count(), ALIAS)
            .group_by(Column::ChannelId)
            .order_by_desc(Expr::col(Alias::new(ALIAS)))
            .into_tuple::<(i64, i64)>()
            .all(self.0.inner())
            .await?
            .into_iter()
            .map(|(channel_id, count)| (ChannelId::new(channel_id as u64), count as u64))
            .collect())
    }

    /// Get user statistics for a guild
    async fn get_user_stats(
        &self,
        guild_id: GuildId,
        channel_ids: Option<&[ChannelId]>,
        from: Option<impl Into<DateTime<FixedOffset>>>,
        to: Option<impl Into<DateTime<FixedOffset>>>,
    ) -> Result<Vec<(UserId, u64)>, DbErr> {
        use sea_orm::sea_query::{Alias, Expr};

        const ALIAS: &str = "message_count";
        Ok(Entity::find()
            .select_only()
            .column(Column::UserId)
            .filter(Column::GuildId.eq(guild_id.get() as i64))
            .filter(channel_ids.map_or(SimpleExpr::Value(true.into()), |c| {
                Column::ChannelId.is_in(c.iter().map(|id| id.get() as i64))
            }))
            .filter(from.map_or(SimpleExpr::Value(true.into()), |f| {
                Column::Timestamp.gte(f.into())
            }))
            .filter(to.map_or(SimpleExpr::Value(true.into()), |t| {
                Column::Timestamp.lt(t.into())
            }))
            .column_as(Column::MessageId.count(), ALIAS)
            .group_by(Column::UserId)
            .order_by_desc(Expr::col(Alias::new(ALIAS)))
            .into_tuple::<(i64, i64)>()
            .all(self.0.inner())
            .await?
            .into_iter()
            .map(|(user_id, count)| (UserId::new(user_id as u64), count as u64))
            .collect())
    }

    /// Get message records for a specific user in a guild
    async fn get_user_messages(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<MessageRecord>, DbErr> {
        Entity::find()
            .filter(
                Column::UserId
                    .eq(user_id.get() as i64)
                    .and(Column::GuildId.eq(guild_id.get() as i64)),
            )
            .order_by_desc(Column::Timestamp)
            .all(self.0.inner())
            .await
    }

    /// Clear all message data (dangerous operation)
    async fn nuke(&self) -> Result<(), DbErr> {
        Entity::delete_many().exec(self.0.inner()).await?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use migration::{Migrator, MigratorTrait, SchemaManager};

    use super::*;
    use crate::database::BotDatabase;

    #[tokio::test]
    async fn test_record_message() {
        let db = BotDatabase::new_memory().await.unwrap();
        let migrations = Migrator::migrations();
        let manager = SchemaManager::new(db.inner());
        for migration in migrations {
            migration.up(&manager).await.unwrap();
        }
        let service = db.message();
        let message_id = MessageId::new(1);
        let user_id = UserId::new(123);
        let guild_id = GuildId::new(456);
        let channel_id = ChannelId::new(789);
        let timestamp = Timestamp::now();
        service
            .record(message_id, user_id, guild_id, channel_id, timestamp)
            .await
            .unwrap();
        let user_stats = service
            .get_user_stats(
                guild_id,
                Some(&[channel_id]),
                None::<DateTime<FixedOffset>>,
                None::<DateTime<FixedOffset>>,
            )
            .await
            .unwrap();
        assert_eq!(user_stats.len(), 1);
        assert_eq!(user_stats[0].0, user_id);
        assert_eq!(user_stats[0].1, 1);
        let channel_stats = service
            .get_channel_stats(
                guild_id,
                None::<DateTime<FixedOffset>>,
                None::<DateTime<FixedOffset>>,
            )
            .await
            .unwrap();
        assert_eq!(channel_stats.len(), 1);
        assert_eq!(channel_stats[0].0, channel_id);
        assert_eq!(channel_stats[0].1, 1);
        let activity = service.get_user_activity(user_id, guild_id).await.unwrap();
        assert_eq!(activity.len(), 1);
        assert_eq!(activity[0], timestamp.to_utc());
    }
}
