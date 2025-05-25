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
pub struct BotCfg {
    pub token: String,
    #[serde(skip)]
    pub path: PathBuf,
}

impl BotCfg {
    pub fn read(path: &Path) -> Result<Self, BotError> {
        let cfg: BotCfg = Figment::new()
            .merge(Json::file(path))
            .merge(Env::prefixed("RUST_BOT_"))
            .extract_lossy()
            .whatever_context("Failed to read configuration file")?;
        Ok(cfg)
    }

    pub fn write(&self) -> Result<(), BotError> {
        let json = serde_json::to_string_pretty(self)
            .whatever_context("Failed to serialize configuration to JSON")?;
        std::fs::write(&self.path, json).whatever_context("Failed to write configuration file")?;
        Ok(())
    }
}
