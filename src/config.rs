use arc_swap::ArcSwap;
use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use itertools::Itertools;
use reqwest::Url;
use serde::{Deserialize, Deserializer, Serialize};
use serenity::all::{ChannelId, GuildId, RoleId, UserId};
use snafu::ResultExt;
use std::{
    collections::{HashMap, HashSet},
    path::{Path, PathBuf},
    sync::LazyLock,
    time::Duration,
};

use crate::error::BotError;

pub static BOT_CONFIG: LazyLock<ArcSwap<BotCfg>> = LazyLock::new(|| {
    let args = crate::Args::parse();
    let cfg = BotCfg {
        path: args.config.to_owned(),
        ..BotCfg::read(args.config.as_path()).expect("Failed to read bot configuration")
    };
    ArcSwap::from_pointee(cfg)
});

fn deserialize_tree_hole_map<'de, D>(
    deserializer: D,
) -> Result<HashMap<ChannelId, Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let string_map: HashMap<&str, u64> = HashMap::deserialize(deserializer)?;
    let channel_map = string_map
        .into_iter()
        .map(|(key, value)| {
            let id = key.parse::<u64>().map_err(serde::de::Error::custom)?;
            let dur = Duration::from_secs(value);
            Ok((ChannelId::new(id), dur))
        })
        .try_collect();

    Ok(channel_map?)
}

fn serialize_tree_hole_map<S>(
    map: &HashMap<ChannelId, Duration>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let string_map: HashMap<String, u64> = map
        .iter()
        .map(|(k, v)| (k.to_string(), v.as_secs()))
        .collect();
    string_map.serialize(serializer)
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BotCfg {
    pub token: String,
    pub volunteer_role_id: RoleId,
    pub supervisor_role_id: RoleId,
    pub supervisors_limit: usize,
    pub supervisor_guilds: Vec<GuildId>,
    pub admin_role_ids: Vec<RoleId>,
    pub extra_admin_user_ids: Vec<UserId>,
    pub cookie_endpoint: Option<Url>,
    pub cookie_secret: String,
    #[serde(deserialize_with = "deserialize_tree_hole_map")]
    #[serde(serialize_with = "serialize_tree_hole_map")]
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
        let json = serde_json::to_string_pretty(self)
            .whatever_context::<&str, BotError>("Failed to serialize configuration to JSON")?;
        std::fs::write(&self.path, json).whatever_context("Failed to write configuration file")
    }
}
