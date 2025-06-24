mod active;
mod cookie;
pub mod flush;
mod health;
pub mod supervisors;
mod tree_hole;

use owo_colors::OwoColorize;
use poise::command;
use serenity::all::{ComponentInteraction, FullEvent, Interaction};
use snafu::OptionExt;
use tracing::{error, info};

use crate::{
    config::BOT_CONFIG,
    error::BotError,
    framework::{active::command::active_chart, flush::command::flush_message},
};
use cookie::command::*;
use health::command::*;
use supervisors::{command::*, handle_supervisor_invitation_response};
use tree_hole::command::*;

pub type Context<'a> = poise::Context<'a, Data, BotError>;

pub async fn check_admin(ctx: Context<'_>) -> Result<bool, BotError> {
    let user_id = ctx.author().id;
    if BOT_CONFIG.load().extra_admin_user_ids.contains(&user_id) {
        return Ok(true);
    }
    Ok(ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?
        .roles
        .iter()
        .any(|&id| BOT_CONFIG.load().admin_role_ids.contains(&id)))
}

#[derive(Debug, Default)]
pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, BotError>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {}", error),
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

fn option() -> poise::FrameworkOptions<Data, BotError> {
    poise::FrameworkOptions {
        commands: vec![
            resign_supervisor(),
            invite_supervisor(),
            health(),
            systemd(),
            register(),
            system_info(),
            submit_cookie(),
            current_supervisors(),
            guilds_info(),
            register_tree_hole(),
            unregister_tree_hole(),
            list_tree_holes(),
            flush_message(),
            active_chart(),
        ],
        on_error: |error| {
            Box::pin(async {
                on_error(error).await;
            })
        },
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
        event_handler: |ctx, event, _, _| {
            Box::pin(async move {
                if let FullEvent::InteractionCreate {
                    interaction: Interaction::Component(component),
                } = event
                {
                    handle_component_interaction(ctx, component).await?;
                }
                Ok(())
            })
        },
        ..Default::default()
    }
}

pub fn framework() -> poise::Framework<Data, BotError> {
    poise::Framework::builder()
        .setup(|_, _, _| {
            Box::pin(async move {
                // This is run when the framework is set up
                info!("Framework has been set up!");
                Ok(Default::default())
            })
        })
        .options(option())
        .build()
}

// You'll need to add this to your main event handler
pub async fn handle_component_interaction(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
) -> Result<(), BotError> {
    if interaction.data.custom_id.starts_with("accept_supervisor")
        || interaction.data.custom_id.starts_with("decline_supervisor")
    {
        handle_supervisor_invitation_response(ctx, interaction).await?;
    }
    Ok(())
}
