mod cookie;
mod health;
pub mod supervisors;

use poise::command;
use serenity::all::{ComponentInteraction, FullEvent, Interaction};
use tracing::{error, info};

use crate::error::BotError;
use cookie::command::*;
use health::command::*;
use supervisors::{command::*, handle_supervisor_invitation_response};

pub type Context<'a> = poise::Context<'a, Data, BotError>;

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
            systemd_status(),
            register(),
            system_info(),
            submit_cookie(),
            current_supervisors(),
        ],
        on_error: |error| {
            Box::pin(async {
                on_error(error).await;
            })
        },
        pre_command: |ctx| Box::pin(async move { info!("Invoke Command: {}", ctx.command().name) }),
        event_handler: |ctx, event, _, _| {
            Box::pin(async move {
                match event {
                    FullEvent::InteractionCreate { interaction } => match interaction {
                        Interaction::Component(component) => {
                            handle_component_interaction(ctx, &component).await?;
                        }
                        _ => {}
                    },
                    _ => {}
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
