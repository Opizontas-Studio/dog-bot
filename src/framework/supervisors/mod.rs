use crate::error::BotError;
use command::*;
use invite::{Invite, handle_supervisor_invitation_response};
use serenity::{
    all::{ComponentInteraction, FullEvent, Interaction, UserId},
    futures::lock::Mutex,
};
use std::collections::HashMap;
use tracing::{error, info};

mod command;
mod invite;

pub type Context<'a> = poise::Context<'a, Data, BotError>;

async fn on_error(error: poise::FrameworkError<'_, Data, BotError>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error,);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error while handling error: {}", e)
            }
        }
    }
}

#[derive(Debug, Default)]
pub struct Data {
    pub pending_invitations: Mutex<HashMap<UserId, Invite>>, // Track pending supervisor invitations
}

fn option() -> poise::FrameworkOptions<Data, BotError> {
    poise::FrameworkOptions {
        commands: vec![
            resign_supervisor(),
            test_add_supervisor(),
            invite_supervisor(),
            register_supervisor(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: None,
            ..Default::default()
        },
        on_error: |error| {
            Box::pin(async {
                on_error(error).await;
            })
        },
        pre_command: |ctx| {
            Box::pin(async move { info!("Executing command {}", ctx.command().name) })
        },
        post_command: |ctx| {
            Box::pin(async move { info!("Finished executing command {}", ctx.command().name) })
        },
        skip_checks_for_owners: true,
        event_handler: |ctx, event, _, data| {
            Box::pin(async move {
                match event {
                    FullEvent::InteractionCreate { interaction } => match interaction {
                        Interaction::Component(component) => {
                            handle_component_interaction(ctx, &component, data).await?;
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
                info!("Supervisors framework has been set up!");
                Ok(Data {
                    pending_invitations: Mutex::new(HashMap::new()),
                })
            })
        })
        .options(option())
        .build()
}

// You'll need to add this to your main event handler
pub async fn handle_component_interaction(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), BotError> {
    if interaction.data.custom_id.starts_with("accept_supervisor")
        || interaction.data.custom_id.starts_with("decline_supervisor")
    {
        handle_supervisor_invitation_response(ctx, interaction, data).await?;
    }
    Ok(())
}
