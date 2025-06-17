use bincode::{Decode, Encode};
use chrono::Duration;
use redb::{ReadableTable, TableDefinition, TableError};
use serenity::all::*;

use crate::database::{BotDatabase, codec::Bincode};

const PENDING_FLUSHES: TableDefinition<u64, Bincode<FlushInfo>> =
    TableDefinition::new("pending_flushes");

#[derive(Debug, Clone, Encode, Decode)]
pub struct FlushInfo {
    pub message_id: u64,
    pub notification_id: u64,
    pub channel_id: u64,
    pub toilet: u64,
    pub author: u64,
    pub flusher: u64,
    pub threshold: u64,
}

impl FlushInfo {
    pub fn toilet(&self) -> ChannelId {
        ChannelId::from(self.toilet)
    }
    pub fn flusher(&self) -> UserId {
        UserId::from(self.flusher)
    }
    pub fn message_id(&self) -> MessageId {
        MessageId::from(self.message_id)
    }
    pub fn notification_id(&self) -> MessageId {
        MessageId::from(self.notification_id)
    }
    pub fn channel_id(&self) -> ChannelId {
        ChannelId::from(self.channel_id)
    }
}

impl BotDatabase {
    pub fn has_flush(&self, message: &Message) -> Result<bool, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PENDING_FLUSHES);
        let table = match table {
            Err(TableError::TableDoesNotExist(_)) => {
                // Table does not exist, no flushes
                return Ok(false);
            }
            Err(e) => {
                return Err(e.into());
            }
            Ok(table) => table,
        };
        Ok(table.get(message.id.get())?.is_some())
    }

    pub fn add_flush(
        &self,
        message: &Message,
        notify: &Message,
        flusher: UserId,
        toilet: ChannelId,
        threshold: u64,
    ) -> Result<(), redb::Error> {
        let flush_info = FlushInfo {
            channel_id: message.channel_id.into(),
            message_id: message.id.into(),
            notification_id: notify.id.into(),
            toilet: toilet.into(),
            threshold,
            author: message.author.id.into(),
            flusher: flusher.into(),
        };
        let write_txn = self.db.begin_write()?;

        {
            let mut table = write_txn.open_table(PENDING_FLUSHES)?;
            table.insert(message.id.get(), flush_info.to_owned())?;
            table.insert(notify.id.get(), flush_info)?;
        }

        write_txn.commit()?;
        Ok(())
    }

    pub fn get_flush(&self, message_id: MessageId) -> Result<Option<FlushInfo>, redb::Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PENDING_FLUSHES);
        let table = match table {
            Err(TableError::TableDoesNotExist(_)) => {
                // Table does not exist, no flushes
                return Ok(None);
            }
            Err(e) => {
                return Err(e.into());
            }
            Ok(table) => table,
        };
        if let Some(flush_info) = table.get(message_id.get())? {
            Ok(Some(flush_info.value()))
        } else {
            Ok(None)
        }
    }

    pub fn remove_flush(&self, message_id: MessageId) -> Result<(), redb::Error> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(PENDING_FLUSHES)?;
            let info = table.get(message_id.get())?.map(|v| v.value());
            // remove another info
            if let Some(info) = info {
                table.remove(info.message_id)?;
                table.remove(info.notification_id)?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn clean_flushes(&self, dur: Duration) -> Result<(), redb::Error> {
        let now = Timestamp::now();
        let bound = now
            .checked_sub_signed(dur)
            .map(Timestamp::from)
            .unwrap_or(now);

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(PENDING_FLUSHES)?;
            table.retain(|key, _| {
                let msg_id = MessageId::new(key);
                let msg_timestamp = msg_id.created_at();
                if msg_timestamp < bound {
                    return false;
                }
                true
            })?;
        }
        write_txn.commit()?;
        Ok(())
    }
}
