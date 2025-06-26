use itertools::Itertools;
use redb::{ReadableTable, TableDefinition};
use serenity::all::*;

use crate::database::BotDatabase;

const CHANNEL_DATA: TableDefinition<(u64, u64), u64> = TableDefinition::new("channel_data");

pub struct ChannelQuery<'a>(&'a BotDatabase);

impl BotDatabase {
    pub fn channels(&self) -> ChannelQuery {
        ChannelQuery(self)
    }
}

impl<'a> ChannelQuery<'a> {
    pub fn update(&self, guild_id: GuildId, channel_id: ChannelId) -> Result<(), Box<redb::Error>> {
        let key = (guild_id.get(), channel_id.get());
        let f = move || -> Result<(), redb::Error> {
            let write_txn = self.0.db.begin_write()?;
            {
                let mut table = write_txn.open_table(CHANNEL_DATA)?;
                let count = table.get(key)?.map_or(0, |v| v.value());
                table.insert(key, count + 1)?;
            }
            write_txn.commit()?;
            Ok(())
        };
        Ok(f()?)
    }

    pub fn get_guild(&self, guild_id: GuildId) -> Result<Vec<(ChannelId, u64)>, Box<redb::Error>> {
        let f = move || -> Result<Vec<(ChannelId, u64)>, redb::Error> {
            let read_txn = self.0.db.begin_read()?;
            let table = read_txn.open_table(CHANNEL_DATA)?;
            Ok(table
                .range((guild_id.get(), u64::MIN)..=(guild_id.get(), u64::MAX))?
                .map_ok(|(key, value)| (key.value().1.into(), value.value()))
                .try_collect()?)
        };
        Ok(f()?)
    }

    pub fn nuke(&self) -> Result<(), Box<redb::Error>> {
        let f = move || -> Result<(), redb::Error> {
            let write_txn = self.0.db.begin_write()?;
            write_txn.delete_table(CHANNEL_DATA)?;
            write_txn.commit()?;
            Ok(())
        };
        Ok(f()?)
    }
}
