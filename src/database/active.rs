use bincode::{Decode, Encode};
use chrono::{DateTime, Utc};
use itertools::Itertools;
use redb::{MultimapTableDefinition, ReadableMultimapTable};
use serenity::all::*;

use crate::database::{BotDatabase, codec::Bincode};

const ACTIVE_DATA: MultimapTableDefinition<u64, Bincode<ActiveData>> =
    MultimapTableDefinition::new("active_data");

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ActiveData {
    pub timestamp: Timestamp,
    pub guild_id: GuildId,
}

impl Encode for ActiveData {
    fn encode<E: bincode::enc::Encoder>(
        &self,
        encoder: &mut E,
    ) -> Result<(), bincode::error::EncodeError> {
        Encode::encode(&self.timestamp.to_utc().timestamp(), encoder)?;
        Encode::encode(&self.guild_id.get(), encoder)?;
        Ok(())
    }
}

impl Decode<()> for ActiveData {
    fn decode<D: bincode::de::Decoder>(
        decoder: &mut D,
    ) -> Result<Self, bincode::error::DecodeError> {
        let timestamp_secs = Decode::decode(decoder)?;
        let guild_id = GuildId::new(Decode::decode(decoder)?);
        let timestamp = Timestamp::from_unix_timestamp(timestamp_secs).map_err(|e| {
            bincode::error::DecodeError::OtherString(format!(
                "Invalid timestamp for guild {}: {}",
                guild_id, e
            ))
        })?;
        Ok(ActiveData {
            guild_id,
            timestamp,
        })
    }
}

impl ActiveData {
    pub fn new(guild: GuildId, timestamp: Timestamp) -> Self {
        Self {
            guild_id: guild,
            timestamp: timestamp,
        }
    }
}
impl BotDatabase {
    pub fn actives(&self) -> Actives {
        Actives(self)
    }
}
pub struct Actives<'a>(&'a BotDatabase);

impl<'a> Actives<'a> {
    pub fn insert(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        timestamp: Timestamp,
    ) -> Result<(), redb::Error> {
        let active_data = ActiveData::new(guild_id, timestamp);
        let write_txn = self.0.db.begin_write()?;
        {
            let mut table = write_txn.open_multimap_table(ACTIVE_DATA)?;
            table.insert(user_id.get(), active_data)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn clean(&self, user_id: UserId) -> Result<(), redb::Error> {
        // Remove data older than 1 day
        const TTL: chrono::Duration = chrono::Duration::days(1);
        let now = chrono::Utc::now();
        let bound = now - TTL;

        let write_txn = self.0.db.begin_write()?;
        {
            let mut table = write_txn.open_multimap_table(ACTIVE_DATA)?;
            // Retain only entries that are newer than the bound
            let old_data = table
                .get(user_id.get())?
                .into_iter()
                .filter_map(|result| result.ok())
                .filter(|active_data| active_data.value().timestamp.to_utc() < bound)
                .map(|active_data| active_data.value().to_owned())
                .collect_vec();
            for data in old_data {
                table.remove(user_id.get(), &data)?;
            }
        }
        Ok(())
    }

    pub fn get(
        &self,
        user_id: UserId,
        guild_id: GuildId,
    ) -> Result<Vec<DateTime<Utc>>, redb::Error> {
        // Allow data within 3 days of the current time
        const TOLERANCE: chrono::Duration = chrono::Duration::days(3);
        let read_txn = self.0.db.begin_read()?;
        let table = read_txn.open_multimap_table(ACTIVE_DATA)?;
        let data = table
            .get(user_id.get())?
            .into_iter()
            .filter_map(|result| result.ok())
            .filter(|active_data| active_data.value().guild_id == guild_id)
            .map(|active_data| active_data.value().timestamp.to_utc())
            .sorted()
            .collect_vec();
        // if oldest is older than 3 days, clean
        let now = Utc::now();
        if let Some(oldest) = data.first() {
            if oldest < &(now - TOLERANCE) {
                self.clean(user_id)?;
            }
        }
        Ok(data)
    }
}
