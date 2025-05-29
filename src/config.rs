use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::{Deserialize, Serialize};
use serenity::all::RoleId;
use snafu::ResultExt;

use crate::error::BotError;

pub static BOT_CONFIG: LazyLock<BotCfg> = LazyLock::new(|| {
    let args = crate::Args::parse();
    let mut cfg =
        BotCfg::read(args.config_path.as_path()).expect("Failed to read bot configuration");
    cfg.path = args.config_path;
    cfg
});

#[derive(Deserialize, Serialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct BotCfg {
    pub token: String,
    pub volunteer_role_id: RoleId,
    pub supervisor_role_id: RoleId,
    pub supervisors_limit: usize,
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
