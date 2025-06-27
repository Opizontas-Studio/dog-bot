use chrono::{DateTime, Utc};
use serenity::all::*;
use sqlx::Row;

use crate::database::BotDatabase;

impl BotDatabase {
    pub fn actives(&self) -> Actives {
        Actives(self)
    }
}

pub struct Actives<'a>(&'a BotDatabase);

impl<'a> Actives<'a> {
    pub async fn insert(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        timestamp: Timestamp,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT OR IGNORE INTO active_data (user_id, guild_id, timestamp) VALUES (?, ?, ?)",
        )
        .bind(user_id.get() as i64)
        .bind(guild_id.get() as i64)
        .bind(timestamp.to_utc().timestamp())
        .execute(self.0.pool())
        .await?;
        Ok(())
    }

    pub async fn clean(&self, user_id: UserId) -> Result<(), sqlx::Error> {
        // Remove data older than 1 day
        const TTL: chrono::Duration = chrono::Duration::days(1);
        let now = chrono::Utc::now();
        let bound = (now - TTL).timestamp();

        sqlx::query("DELETE FROM active_data WHERE user_id = ? AND timestamp < ?")
            .bind(user_id.get() as i64)
            .bind(bound)
            .execute(self.0.pool())
            .await?;
        Ok(())
    }

    pub async fn get(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, sqlx::Error> {
        // Allow data within 3 days of the current time
        const TOLERANCE: chrono::Duration = chrono::Duration::days(3);

        let rows = sqlx::query(
            "SELECT timestamp FROM active_data WHERE user_id = ? AND guild_id = ? ORDER BY timestamp"
        )
        .bind(user_id.get() as i64)
        .bind(guild_id.get() as i64)
        .fetch_all(self.0.pool())
        .await?;

        let data: Vec<DateTime<Utc>> = rows
            .into_iter()
            .map(|row| {
                let timestamp: i64 = row.get("timestamp");
                DateTime::from_timestamp(timestamp, 0).unwrap_or_default()
            })
            .collect();

        let now = Utc::now();
        if let Some(oldest) = data.first() {
            if oldest < &(now - TOLERANCE) {
                self.clean(user_id).await?;
            }
        }
        Ok(data)
    }
}
