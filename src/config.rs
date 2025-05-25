use std::{path::Path, sync::LazyLock};

use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Json},
};
use serde::Deserialize;
use snafu::ResultExt;

use crate::error::BotError;

pub static BOT_CONFIG: LazyLock<BotCfg> = LazyLock::new(|| {
    let args = crate::Args::parse();
    BotCfg::read(args.config_path.as_path()).expect("Failed to read bot configuration")
});

#[derive(Deserialize, Debug, Default)]
pub struct BotCfg {
    pub token: String,
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
}
