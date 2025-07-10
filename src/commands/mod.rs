mod cookie;
pub mod flush;
mod help;
mod ping;
mod stats;
mod system;
mod tree_hole;

use std::sync::Arc;

use arc_swap::ArcSwap;
use cookie::*;
use flush::*;
use help::*;
use owo_colors::OwoColorize;
use ping::*;
use poise::{PrefixFrameworkOptions, command};
use snafu::OptionExt;
use stats::*;
use system::*;
use tracing::{error, info};
use tree_hole::*;

use crate::{config::BotCfg, database::BotDatabase, error::BotError};

pub type Context<'a> = poise::Context<'a, Data, BotError>;

pub async fn check_admin(ctx: Context<'_>) -> Result<bool, BotError> {
    let user_id = ctx.author().id;
    if ctx
        .data()
        .cfg
        .load()
        .extra_admin_user_ids
        .contains(&user_id)
    {
        return Ok(true);
    }
    Ok(ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?
        .roles
        .iter()
        .any(|&id| ctx.data().cfg.load().admin_role_ids.contains(&id)))
}

#[derive(Debug)]
pub struct Data {
    db: BotDatabase,
    cfg: Arc<ArcSwap<BotCfg>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, BotError>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {error}"),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[command(prefix_command, owners_only)]
async fn register(ctx: Context<'_>) -> Result<(), BotError> {
    Ok(poise::builtins::register_application_commands_buttons(ctx).await?)
}

fn option(cfg: &ArcSwap<BotCfg>) -> poise::FrameworkOptions<Data, BotError> {
    poise::FrameworkOptions {
        commands: vec![
            guilds_info(),
            register(),
            system_info(),
            submit_cookie(),
            register_tree_hole(),
            unregister_tree_hole(),
            list_tree_holes(),
            flush_message(),
            channel_stats(),
            user_stats(),
            ping(),
            help(),
        ],
        prefix_options: PrefixFrameworkOptions {
            prefix: "!".to_string().into(),
            ..Default::default()
        },
        on_error: |error| {
            Box::pin(async {
                on_error(error).await;
            })
        },
        owners: cfg
            .load()
            .extra_owners
            .iter()
            .map(|id| id.to_owned())
            .collect(),
        skip_checks_for_owners: true,
        pre_command: |ctx| {
            Box::pin(async move {
                info!(
                    "Command: {}\tUser: {}\tGuild: {}",
                    ctx.command().name.green(),
                    ctx.author().name.green(),
                    ctx.guild()
                        .map(|g| g.name.to_owned())
                        .unwrap_or("DM".to_string())
                        .green()
                )
            })
        },
        ..Default::default()
    }
}

pub fn framework(db: BotDatabase, cfg: Arc<ArcSwap<BotCfg>>) -> poise::Framework<Data, BotError> {
    poise::Framework::builder()
        .options(option(&cfg))
        .setup(|_, _, _| {
            Box::pin(async move {
                // This is run when the framework is set up
                info!("Framework has been set up!");
                Ok(Data { db, cfg })
            })
        })
        .build()
}
