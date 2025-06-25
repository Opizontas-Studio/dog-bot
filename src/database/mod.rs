mod active;
mod codec;
mod flush;
mod invite;
use std::{path::Path, sync::LazyLock};

use bincode::{
    config::standard,
    serde::{decode_from_slice, encode_to_vec},
};
use chrono::{DateTime, Utc};
use clap::Parser;
use redb::{Database, Error, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use serenity::all::{MessageId, UserId};

use crate::{Args, error::BotError};

pub static DB: LazyLock<BotDatabase> =
    LazyLock::new(|| BotDatabase::new(Args::parse().redb).expect("Failed to initialize database"));

// Table definitions
const SUPERVISORS_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("supervisors");
const POLLS_TABLE: TableDefinition<u64, &[u8]> = TableDefinition::new("polls");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supervisor {
    pub id: UserId,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub polls: Vec<MessageId>,
    #[serde(default)]
    pub since: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PollStage {
    Proposal,
    Outdated,
    Polling,
    Approved,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Poll {
    pub id: MessageId,
    pub proposer: UserId,
    pub stage: PollStage,
    pub signs_needed: u64,
    pub approves_needed: u64,
    pub approve_ratio_needed: f64,
    pub signatures: Vec<UserId>,
    pub approves: Vec<UserId>,
    pub rejects: Vec<UserId>,
}

pub struct BotDatabase {
    db: Database,
}

impl BotDatabase {
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Box<Error>> {
        let db = Database::builder()
            .create_with_file_format_v3(true)
            .create(path)
            .map_err(Error::from)?;
        Ok(BotDatabase { db })
    }

    // Supervisor operations
    pub fn add_supervisor(&self, supervisor: Supervisor) -> Result<(), BotError> {
        let write_txn = self.db.begin_write().map_err(redb::Error::from)?;
        {
            let mut table = write_txn
                .open_table(SUPERVISORS_TABLE)
                .map_err(Error::from)?;
            let serialized = encode_to_vec(&supervisor, standard())?;
            table
                .insert(supervisor.id.get(), serialized.as_slice())
                .map_err(Error::from)?;
        }
        write_txn.commit().map_err(Error::from)?;
        Ok(())
    }

    pub fn get_supervisor(&self, id: UserId) -> Result<Option<Supervisor>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn
            .open_table(SUPERVISORS_TABLE)
            .map_err(Error::from)?;

        if let Some(data) = table.get(id.get()).map_err(Error::from)? {
            let (supervisor, _) = decode_from_slice(data.value(), standard())?;
            Ok(Some(supervisor))
        } else {
            Ok(None)
        }
    }

    pub fn update_supervisor(&self, supervisor: Supervisor) -> Result<(), BotError> {
        self.add_supervisor(supervisor) // Same as add since we're replacing
    }

    pub fn get_active_supervisors(&self) -> Result<Vec<Supervisor>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn
            .open_table(SUPERVISORS_TABLE)
            .map_err(Error::from)?;
        let mut active_supervisors = Vec::new();

        let iter = table.iter().map_err(Error::from)?;
        for entry in iter {
            let (_, data) = entry.map_err(Error::from)?;
            let (supervisor, _): (Supervisor, _) = decode_from_slice(data.value(), standard())?;
            if supervisor.active {
                active_supervisors.push(supervisor);
            }
        }

        Ok(active_supervisors)
    }

    pub fn deactivate_supervisor(&self, id: UserId) -> Result<(), BotError> {
        if let Some(mut supervisor) = self.get_supervisor(id)? {
            supervisor.active = false;
            self.update_supervisor(supervisor)?;
        }
        Ok(())
    }

    pub fn add_poll_to_supervisor(
        &self,
        supervisor_id: UserId,
        poll_id: MessageId,
    ) -> Result<(), BotError> {
        if let Some(mut supervisor) = self.get_supervisor(supervisor_id)? {
            if !supervisor.polls.contains(&poll_id) {
                supervisor.polls.push(poll_id);
                self.update_supervisor(supervisor)?;
            }
        }
        Ok(())
    }

    // Poll operations
    pub fn add_poll(&self, poll: Poll) -> Result<(), BotError> {
        let write_txn = self.db.begin_write().map_err(Error::from)?;
        {
            let mut table = write_txn.open_table(POLLS_TABLE).map_err(Error::from)?;
            let serialized = encode_to_vec(&poll, standard())?;
            table
                .insert(poll.id.get(), serialized.as_slice())
                .map_err(Error::from)?;
        }
        write_txn.commit().map_err(Error::from)?;

        // Add poll to proposer's poll list
        self.add_poll_to_supervisor(poll.proposer, poll.id)?;

        Ok(())
    }

    pub fn get_poll(&self, id: MessageId) -> Result<Option<Poll>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn.open_table(POLLS_TABLE).map_err(Error::from)?;

        if let Some(data) = table.get(id.get()).map_err(Error::from)? {
            let (poll, _) = decode_from_slice(data.value(), standard())?;
            Ok(Some(poll))
        } else {
            Ok(None)
        }
    }

    pub fn update_poll(&self, poll: Poll) -> Result<(), BotError> {
        self.add_poll(poll) // Same as add since we're replacing
    }

    pub fn get_polls_by_stage(&self, stage: PollStage) -> Result<Vec<Poll>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn.open_table(POLLS_TABLE).map_err(Error::from)?;
        let mut polls = Vec::new();

        let iter = table.iter().map_err(Error::from)?;
        for entry in iter {
            let (_, data) = entry.map_err(Error::from)?;
            let (poll, _): (Poll, _) = decode_from_slice(data.value(), standard())?;
            if poll.stage == stage {
                polls.push(poll);
            }
        }

        Ok(polls)
    }

    pub fn sign_poll(&self, poll_id: MessageId, supervisor_id: UserId) -> Result<bool, BotError> {
        if let Some(mut poll) = self.get_poll(poll_id)? {
            // Check if supervisor is active
            if let Some(supervisor) = self.get_supervisor(supervisor_id)? {
                if !supervisor.active {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            // Check if already signed
            if poll.signatures.contains(&supervisor_id) {
                return Ok(false);
            }

            // Add signature
            poll.signatures.push(supervisor_id);

            // Check if enough signatures to move to polling stage
            if poll.signatures.len() as u64 >= poll.signs_needed
                && poll.stage == PollStage::Proposal
            {
                poll.stage = PollStage::Polling;
            }

            self.update_poll(poll)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn vote_poll(
        &self,
        poll_id: MessageId,
        supervisor_id: UserId,
        approve: bool,
    ) -> Result<bool, BotError> {
        if let Some(mut poll) = self.get_poll(poll_id)? {
            // Check if poll is in polling stage
            if poll.stage != PollStage::Polling {
                return Ok(false);
            }

            // Check if supervisor is active
            if let Some(supervisor) = self.get_supervisor(supervisor_id)? {
                if !supervisor.active {
                    return Ok(false);
                }
            } else {
                return Ok(false);
            }

            // Remove from both lists first (in case changing vote)
            poll.approves.retain(|&id| id != supervisor_id);
            poll.rejects.retain(|&id| id != supervisor_id);

            // Add to appropriate list
            if approve {
                poll.approves.push(supervisor_id);
            } else {
                poll.rejects.push(supervisor_id);
            }

            // Check if voting is complete
            let total_votes = poll.approves.len() + poll.rejects.len();
            let approve_count = poll.approves.len() as u64;
            let approve_ratio = if total_votes > 0 {
                poll.approves.len() as f64 / total_votes as f64
            } else {
                0.0
            };

            if approve_count >= poll.approves_needed && approve_ratio >= poll.approve_ratio_needed {
                poll.stage = PollStage::Approved;
            } else if total_votes > 0 && approve_ratio < poll.approve_ratio_needed {
                // Check if it's impossible to reach the required ratio
                let active_supervisors = self.get_active_supervisors()?;
                let max_possible_approves =
                    approve_count + (active_supervisors.len() - total_votes) as u64;
                let max_possible_ratio =
                    max_possible_approves as f64 / active_supervisors.len() as f64;

                if max_possible_ratio < poll.approve_ratio_needed {
                    poll.stage = PollStage::Rejected;
                }
            }

            self.update_poll(poll)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn get_supervisor_polls(&self, supervisor_id: UserId) -> Result<Vec<Poll>, BotError> {
        if let Some(supervisor) = self.get_supervisor(supervisor_id)? {
            let mut polls = Vec::new();
            for poll_id in supervisor.polls {
                if let Some(poll) = self.get_poll(poll_id)? {
                    polls.push(poll);
                }
            }
            Ok(polls)
        } else {
            Ok(Vec::new())
        }
    }

    pub fn delete_poll(&self, poll_id: u64) -> Result<(), Box<Error>> {
        let f = move || -> Result<(), redb::Error> {
            let write_txn = self.db.begin_write()?;
            {
                let mut table = write_txn.open_table(POLLS_TABLE)?;
                table.remove(poll_id)?;
            }
            write_txn.commit()?;
            Ok(())
        };
        Ok(f()?)
    }

    pub fn get_all_supervisors(&self) -> Result<Vec<Supervisor>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn
            .open_table(SUPERVISORS_TABLE)
            .map_err(Error::from)?;
        let mut supervisors = Vec::new();

        let iter = table.iter().map_err(Error::from)?;
        for entry in iter {
            let (_, data) = entry.map_err(Error::from)?;
            let (supervisor, _): (Supervisor, _) = decode_from_slice(data.value(), standard())?;
            supervisors.push(supervisor);
        }

        Ok(supervisors)
    }

    pub fn get_all_polls(&self) -> Result<Vec<Poll>, BotError> {
        let read_txn = self.db.begin_read().map_err(Error::from)?;
        let table = read_txn.open_table(POLLS_TABLE).map_err(Error::from)?;
        let mut polls = Vec::new();

        let iter = table.iter().map_err(Error::from)?;
        for entry in iter {
            let (_, data) = entry.map_err(Error::from)?;
            let (poll, _): (Poll, _) = decode_from_slice(data.value(), standard())?;
            polls.push(poll);
        }

        Ok(polls)
    }
}

// Helper functions for creating new instances
impl Supervisor {
    pub fn new(id: UserId) -> Self {
        Self {
            id,
            active: true,
            polls: Vec::new(),
            since: Utc::now(),
        }
    }
}

impl Poll {
    pub fn new(
        id: MessageId,
        proposer: UserId,
        signs_needed: u64,
        approves_needed: u64,
        approve_ratio_needed: f64,
    ) -> Self {
        Self {
            id,
            proposer,
            stage: PollStage::Proposal,
            signs_needed,
            approves_needed,
            approve_ratio_needed,
            signatures: vec![proposer], // Proposer automatically signs
            approves: Vec::new(),
            rejects: Vec::new(),
        }
    }
}
