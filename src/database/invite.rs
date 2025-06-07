use itertools::Itertools;
use redb::{Error, ReadableTable, TableDefinition};
use serenity::all::{GuildId, Message, UserId};

use super::codec::Bincode;
use crate::database::BotDatabase;
use crate::framework::supervisors::Invite;

const PENDING_INVITATIONS: TableDefinition<Bincode<UserId>, Bincode<Invite>> =
    TableDefinition::new("pending_invitations");

impl BotDatabase {
    pub fn insert_invite(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        message: Message,
    ) -> Result<(), Error> {
        let invite = Invite { guild_id, message };
        let write_txn = self.db.begin_write()?;

        {
            let mut table = write_txn.open_table(PENDING_INVITATIONS)?;
            table.insert(user_id, invite)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn remove_invite(&self, user_id: UserId) -> Result<Option<Invite>, Error> {
        let write_txn = self.db.begin_write()?;
        let invite = {
            let mut table = write_txn.open_table(PENDING_INVITATIONS)?;
            let invite = table.remove(user_id).map_err(Error::from)?;
            invite.map(|b| b.value())
        };
        write_txn.commit()?;
        Ok(invite)
    }

    pub fn pending_users(&self) -> Result<Vec<UserId>, Error> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(PENDING_INVITATIONS)?;
        Ok(table
            .iter()?
            .map(|result| result.map(|(key, _)| key.value()))
            .try_collect()?)
    }
}
