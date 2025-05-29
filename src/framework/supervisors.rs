use crate::{config::BOT_CONFIG, error::BotError};
use poise::command;
use rand::seq::IndexedRandom;
use serenity::{
    all::{
        ButtonStyle, ComponentInteraction, CreateButton, CreateInteractionResponse,
        CreateInteractionResponseMessage, CreateMessage, FullEvent, GuildId, Interaction, Member,
        UserId,
    },
    futures::lock::Mutex,
};
use snafu::OptionExt;
use std::collections::HashMap;
use tracing::{error, info, warn};

type Context<'a> = poise::Context<'a, Data, BotError>;

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
    pub pending_invitations: Mutex<HashMap<UserId, GuildId>>, // Track pending supervisor invitations
}

/// Get all members with the volunteer role who aren't already supervisors
async fn get_eligible_volunteers(ctx: Context<'_>) -> Result<Vec<Member>, BotError> {
    let guild = ctx
        .guild()
        .whatever_context::<&str, BotError>("Failed to get guild information")?;
    let members = guild.members.values().cloned().collect::<Vec<_>>();
    let volunteer_role_id = BOT_CONFIG.volunteer_role_id;
    let supervisor_role_id = BOT_CONFIG.supervisor_role_id;
    Ok(members
        .into_iter()
        .filter(|member| {
            member.roles.contains(&volunteer_role_id) && !member.roles.contains(&supervisor_role_id)
        })
        .collect())
}

/// Send supervisor invitation DM to a user
async fn send_supervisor_invitation(ctx: Context<'_>, target_user: UserId) -> Result<(), BotError> {
    let user = target_user.to_user(ctx).await?;
    let guild_id = ctx
        .guild_id()
        .whatever_context::<&str, BotError>("No guild context available")?;

    let accept_button = CreateButton::new("accept_supervisor")
        .label("Accept")
        .style(ButtonStyle::Success);

    let decline_button = CreateButton::new("decline_supervisor")
        .label("Decline")
        .style(ButtonStyle::Danger);

    let message = CreateMessage::new()
        .content("🎉 **Supervisor Invitation**\n\nYou've been randomly selected to become a supervisor! This is an opportunity to help manage and support the community.\n\nWould you like to accept this role?")
        .button(accept_button)
        .button(decline_button);

    match user.direct_message(ctx, message).await {
        Ok(_) => {
            info!("Sent supervisor invitation to {}", user.name);
            // Add to pending invitations
            let mut pending = ctx.data().pending_invitations.lock().await;
            pending.insert(user.id, guild_id);
        }
        Err(e) => {
            warn!("Failed to send DM to {}: {}", user.name, e);
            return Err(e.into()); // Convert to BotError
        }
    }

    Ok(())
}

/// Handle button interactions for supervisor invitations
pub async fn handle_supervisor_invitation_response(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
    data: &Data,
) -> Result<(), BotError> {
    let user_id = interaction.user.id;

    // Check if this user has a pending invitation
    let mut pending = data.pending_invitations.lock().await;
    let Some(guild_id) = pending.get(&user_id).cloned() else {
        // No pending invitation for this user
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❌ **Error**\n\nYou don't have a pending supervisor invitation.")
                .ephemeral(true),
        );
        interaction.create_response(ctx, response).await?;
        return Ok(());
    };

    match interaction.data.custom_id.as_str() {
        "accept_supervisor" => {
            handle_accept_supervisor(ctx, interaction, user_id, guild_id).await?;
        }
        "decline_supervisor" => {
            handle_decline_supervisor(ctx, interaction).await?;
        }
        _ => return Ok(()), // Not our button
    }

    // Remove from pending invitations
    pending.remove(&user_id);
    Ok(())
}

async fn handle_accept_supervisor(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
    user_id: UserId,
    guild_id: GuildId,
) -> Result<(), BotError> {
    let Ok(member) = guild_id.member(ctx, user_id).await else {
        return Ok(());
    };
    // check current number of supervisors
    let current_supervisors = {
        let guild = guild_id
            .to_guild_cached(ctx)
            .whatever_context::<&str, BotError>("Failed to get guild information")?;
        let supervisor_role_id = BOT_CONFIG.supervisor_role_id;
        guild
            .members
            .values()
            .filter(|m| m.roles.contains(&supervisor_role_id))
            .count()
    };
    if current_supervisors >= BOT_CONFIG.supervisors_limit {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❌ **Error**\n\nSorry, you are late! We already have enough supervisors for now. You can still help out as a volunteer!")
                .ephemeral(true)
        );
        interaction.create_response(ctx, response).await?;
        return Ok(());
    }

    if let Err(e) = member.add_role(ctx, BOT_CONFIG.supervisor_role_id).await {
        error!(
            "Failed to add supervisor role to {}: {}",
            interaction.user.name, e
        );
        let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ **Error**\n\nSorry, there was an error adding the supervisor role. Please contact an administrator.")
                    .ephemeral(true)
            );
        interaction.create_response(ctx, response).await?;
    }

    info!("{} accepted supervisor invitation", interaction.user.name);
    let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("✅ **Congratulations!**\n\nYou are now a supervisor! Welcome to the team. You can use `/resign_supervisor` if you ever want to step down from this role.")
                    .ephemeral(true)
            );
    interaction.create_response(ctx, response).await?;

    Ok(())
}

async fn handle_decline_supervisor(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
) -> Result<(), BotError> {
    info!("{} declined supervisor invitation", interaction.user.name);
    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .content("👍 **No problem!**\n\nYou've declined the supervisor invitation. You may receive another invitation in the future if more supervisors are needed.")
            .ephemeral(true)
    );
    interaction.create_response(ctx, response).await?;
    Ok(())
}

/// Quits the current user from being a supervisor and potentially invites a new one.
#[command(slash_command, guild_only, owners_only)]
async fn resign_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    let role_id = BOT_CONFIG.supervisor_role_id;

    if !member.roles.contains(&role_id) {
        info!("{} is not a supervisor", ctx.author().name);
        ctx.say("❌ You are not a supervisor!").await?;
        return Ok(());
    }

    // Remove role from member
    member.remove_role(ctx, role_id).await?;
    info!("{} has resigned from being a supervisor", ctx.author().name);
    ctx.say("You have resigned from being a supervisor.")
        .await?;

    Ok(())
}

async fn random_invite_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    // Try to invite a random volunteer to become supervisor
    let volunteers = match get_eligible_volunteers(ctx).await {
        Ok(volunteers) => volunteers,
        Err(e) => {
            error!("Failed to get eligible volunteers: {}", e);
            ctx.say("✅ You have resigned from being a supervisor! However, we couldn't check for available volunteers to invite.").await?;
            return Ok(());
        }
    };

    if volunteers.is_empty() {
        ctx.say("✅ You have resigned from being a supervisor! No volunteers are currently available to invite.").await?;
        return Ok(());
    }

    // Filter out users with pending invitations
    let available_volunteers = {
        let pending = ctx.data().pending_invitations.lock().await;
        volunteers
            .into_iter()
            .filter(|member| !pending.contains_key(&member.user.id))
            .collect::<Vec<_>>()
    };

    if available_volunteers.is_empty() {
        ctx.say("✅ You have resigned from being a supervisor! All eligible volunteers already have pending invitations.").await?;
        return Ok(());
    }

    // Randomly select a volunteer
    let selected_volunteer = {
        let mut rng = rand::rng();
        available_volunteers.choose(&mut rng)
    };

    let Some(selected_volunteer) = selected_volunteer else {
        ctx.say("✅ You have resigned from being a supervisor! No volunteers are currently available to invite.").await?;
        return Ok(());
    };

    let volunteer_id = selected_volunteer.user.id;
    match send_supervisor_invitation(ctx, volunteer_id).await {
        Ok(_) => {
            ctx.say("✅ You have resigned from being a supervisor! A random volunteer has been invited to take your place.").await?;
        }
        Err(e) => {
            warn!("Failed to send invitation: {}", e);
            ctx.say("✅ You have resigned from being a supervisor! However, we couldn't send an invitation to a replacement.").await?;
        }
    }
    Ok(())
}

#[command(prefix_command, guild_only, owners_only)]
async fn test_add_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    member.add_role(ctx, BOT_CONFIG.supervisor_role_id).await?;
    info!("{} has been added as a supervisor", ctx.author().name);
    ctx.say("You have been added as a supervisor!").await?;
    Ok(())
}

/// Manually invite a volunteer to become supervisor (for testing/admin use)
#[command(slash_command, guild_only, owners_only)]
async fn invite_supervisor(ctx: Context<'_>, member: Member) -> Result<(), BotError> {
    let volunteer_id = member.user.id;
    let volunteer_name = &member.user.name;

    match send_supervisor_invitation(ctx, volunteer_id).await {
        Ok(_) => {
            ctx.say(format!(
                "✅ Supervisor invitation sent to **{}**!",
                volunteer_name
            ))
            .await?;
        }
        Err(e) => {
            warn!("Failed to send invitation: {}", e);
            ctx.say(format!(
                "❌ Failed to send invitation to **{}**. They may have DMs disabled.",
                volunteer_name
            ))
            .await?;
        }
    }

    Ok(())
}

#[command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), BotError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

fn option() -> poise::FrameworkOptions<Data, BotError> {
    poise::FrameworkOptions {
        commands: vec![
            resign_supervisor(),
            test_add_supervisor(),
            invite_supervisor(),
            register(),
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
