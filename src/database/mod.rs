mod active;
mod channel;
mod codec;
mod flush;
use std::{path::Path, sync::LazyLock};

use chrono::{DateTime, Utc};
use clap::Parser;
use redb::{Database, Error};
use serde::{Deserialize, Serialize};
use serenity::all::{MessageId, UserId};

use crate::Args;

pub static DB: LazyLock<BotDatabase> =
    LazyLock::new(|| BotDatabase::new(Args::parse().redb).expect("Failed to initialize database"));

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
}
