use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use arc_swap::ArcSwap;
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::{DurationSeconds, serde_as};
use serenity::all::{ChannelId, GuildId, RoleId, UserId};
use snafu::ResultExt;

use crate::error::BotError;

pub static BOT_CONFIG: LazyLock<ArcSwap<BotCfg>> = LazyLock::new(|| {
    let args = crate::Args::parse();
    let cfg = BotCfg {
        path: args.config.to_owned(),
        ..BotCfg::read(args.config.as_path()).expect("Failed to read bot configuration")
    };
    ArcSwap::from_pointee(cfg)
});

#[serde_as]
#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotCfg {
    pub token: String,
    pub supervisor_guilds: Vec<GuildId>,
    pub admin_role_ids: Vec<RoleId>,
    pub extra_admin_user_ids: Vec<UserId>,
    pub cookie_endpoint: Option<Url>,
    pub cookie_secret: String,
    #[serde_as(as = "Vec<(_, DurationSeconds)>")]
    pub tree_holes: HashMap<ChannelId, Duration>,
    pub toilets: HashSet<ChannelId>,
    pub extra_owners: HashSet<UserId>,
    #[serde(skip)]
    pub path: PathBuf,
}

impl BotCfg {
    pub fn read(path: &Path) -> Result<Self, BotError> {
        Figment::new()
            .merge(Json::file(path))
            .merge(Env::prefixed("RUST_BOT_"))
            .extract_lossy()
            .whatever_context("Failed to read bot configuration")
    }

    pub fn write(&self) -> Result<(), BotError> {
        let json = serenity::json::to_string_pretty(self)
            .whatever_context::<&str, BotError>("Failed to serialize configuration to JSON")?;
        std::fs::write(&self.path, json).whatever_context("Failed to write configuration file")
    }
}
