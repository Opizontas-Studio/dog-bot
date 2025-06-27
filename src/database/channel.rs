use serenity::all::*;
use sqlx::Row;

use crate::database::BotDatabase;

pub struct ChannelQuery<'a>(&'a BotDatabase);

impl BotDatabase {
    pub fn channels(&self) -> ChannelQuery {
        ChannelQuery(self)
    }
}

impl<'a> ChannelQuery<'a> {
    pub async fn update(
        &self,
        guild_id: GuildId,
        channel_id: ChannelId,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO channel_data (guild_id, channel_id, message_count) 
            VALUES (?, ?, 1)
            ON CONFLICT(guild_id, channel_id) 
            DO UPDATE SET message_count = message_count + 1
            "#,
        )
        .bind(guild_id.get() as i64)
        .bind(channel_id.get() as i64)
        .execute(self.0.pool())
        .await?;
        Ok(())
    }

    pub async fn get_guild(&self, guild_id: GuildId) -> Result<Vec<(ChannelId, u64)>, sqlx::Error> {
        let rows = sqlx::query(
            "SELECT channel_id, message_count FROM channel_data WHERE guild_id = ? ORDER BY message_count DESC"
        )
        .bind(guild_id.get() as i64)
        .fetch_all(self.0.pool())
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| {
                let channel_id: i64 = row.get("channel_id");
                let message_count: i64 = row.get("message_count");
                (ChannelId::new(channel_id as u64), message_count as u64)
            })
            .collect())
    }

    pub async fn nuke(&self) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM channel_data")
            .execute(self.0.pool())
            .await?;
        Ok(())
    }
}
