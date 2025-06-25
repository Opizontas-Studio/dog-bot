use bincode::{Decode, Encode, enc::Encoder, error::*};
use itertools::Itertools;
use redb::{Error, ReadableTable, TableDefinition};
use serenity::all::{ChannelId, GuildId, MessageId, UserId};

use super::codec::Bincode;
use crate::database::BotDatabase;
use crate::framework::supervisors::Invite;

const PENDING_INVITATIONS: TableDefinition<u64, Bincode<Invite>> =
    TableDefinition::new("pending_invitations");

impl Encode for Invite {
    fn encode<E: Encoder>(&self, encoder: &mut E) -> Result<(), EncodeError> {
        Encode::encode(&self.guild_id.get(), encoder)?;
        Encode::encode(&self.channel_id.get(), encoder)?;
        Encode::encode(&self.message_id.get(), encoder)?;
        Ok(())
    }
}

impl Decode<()> for Invite {
    fn decode<D: bincode::de::Decoder>(decoder: &mut D) -> Result<Self, DecodeError> {
        let guild_id = GuildId::new(Decode::decode(decoder)?);
        let channel_id = ChannelId::new(Decode::decode(decoder)?);
        let message_id = MessageId::new(Decode::decode(decoder)?);
        Ok(Invite {
            guild_id,
            channel_id,
            message_id,
        })
    }
}

pub struct Invites<'a>(&'a BotDatabase);

impl BotDatabase {
    pub fn invites(&self) -> Invites {
        Invites(self)
    }
}

impl<'a> Invites<'a> {
    pub fn insert(
        &self,
        user_id: UserId,
        guild_id: GuildId,
        channel_id: ChannelId,
        message_id: MessageId,
    ) -> Result<(), Box<Error>> {
        let invite = Invite {
            guild_id,
            channel_id,
            message_id,
        };
        let f = move || -> Result<(), Error> {
            let write_txn = self.0.db.begin_write()?;
            {
                let mut table = write_txn.open_table(PENDING_INVITATIONS)?;
                table.insert(user_id.get(), invite)?;
            }
            write_txn.commit()?;
            Ok(())
        };
        Ok(f()?)
    }

    pub fn remove(&self, user_id: UserId) -> Result<Option<Invite>, Box<Error>> {
        let f = move || -> Result<Option<Invite>, Error> {
            let write_txn = self.0.db.begin_write()?;
            let invite = {
                let mut table = write_txn.open_table(PENDING_INVITATIONS)?;
                let invite = table.remove(user_id.get())?;
                invite.map(|b| b.value())
            };
            write_txn.commit()?;
            Ok(invite)
        };
        Ok(f()?)
    }

    pub fn pending(&self) -> Result<Vec<UserId>, Box<Error>> {
        let f = move || -> Result<Vec<UserId>, Error> {
            let read_txn = self.0.db.begin_read()?;
            let table = read_txn.open_table(PENDING_INVITATIONS)?;
            Ok(table
                .iter()?
                .map(|result| result.map(|(key, _)| key.value().into()))
                .try_collect()?)
        };
        Ok(f()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::BotDatabase;

    #[test]
    fn test_insert_and_remove_invite() {
        let file = tempfile::NamedTempFile::new().unwrap();
        let db = BotDatabase::new(file.path()).unwrap();
        let user_id = UserId::new(123456789);
        let guild_id = GuildId::new(987654321);
        let channel_id = ChannelId::new(1122334455);
        let message_id = MessageId::new(5566778899);

        db.invites()
            .insert(user_id, guild_id, channel_id, message_id)
            .unwrap();
        assert!(db.invites().pending().unwrap().contains(&user_id));
        let invite = db.invites().remove(user_id).unwrap();

        assert!(invite.is_some());
        assert_eq!(invite.unwrap().guild_id, guild_id);
    }
}
