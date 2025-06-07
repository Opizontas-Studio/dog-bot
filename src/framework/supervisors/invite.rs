use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serenity::all::{
    ButtonStyle, ComponentInteraction, CreateButton, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, GuildId, Member, Message, UserId,
};
use snafu::OptionExt;
use tracing::{error, info, warn};

use crate::{config::BOT_CONFIG, database::DB, error::BotError};

use super::super::{Context, Data};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub guild_id: GuildId,
    pub message: Message,
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

/// Handle button interactions for supervisor invitations
pub async fn handle_supervisor_invitation_response(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
) -> Result<(), BotError> {
    let user_id = interaction.user.id;

    // Check if this user has a pending invitation
    let Some(invite) = DB.remove_invite(user_id)? else {
        // No pending invitation for this user
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❌ **Error**\n\nYou don't have a pending supervisor invitation.")
                .ephemeral(true),
        );
        interaction.create_response(ctx, response).await?;
        return Ok(());
    };
    // delete the original message
    if let Err(e) = invite.message.delete(ctx).await {
        error!("Failed to delete invitation message: {}", e);
    }

    match interaction.data.custom_id.as_str() {
        "accept_supervisor" => {
            handle_accept_supervisor(ctx, interaction, user_id, invite.guild_id).await?;
        }
        "decline_supervisor" => {
            handle_decline_supervisor(ctx, interaction).await?;
        }
        _ => return Ok(()), // Not our button
    }
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
        let pending = DB.pending_users()?;
        volunteers
            .into_iter()
            .filter(|member| !pending.contains(&member.user.id))
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
pub async fn send_supervisor_invitation(
    ctx: Context<'_>,
    target_user: UserId,
) -> Result<(), BotError> {
    let user = target_user.to_user(ctx).await?;
    let guild_id = ctx
        .guild_id()
        .whatever_context::<&str, BotError>("No guild context available")?;

    let accept_button = CreateButton::new("accept_supervisor")
        .label("✅")
        .style(ButtonStyle::Success);

    let decline_button = CreateButton::new("decline_supervisor")
        .label("❌")
        .style(ButtonStyle::Danger);

    let message = CreateMessage::new()
        .content("🎉 **Supervisor Invitation**\n\nYou've been randomly selected to become a supervisor! This is an opportunity to help manage and support the community.\n\nWould you like to accept this role?")
        .button(accept_button)
        .button(decline_button);

    match user.direct_message(ctx, message).await {
        Ok(m) => {
            info!("Sent supervisor invitation to {}", user.name);
            // Add to pending invitations
            DB.insert_invite(target_user, guild_id, m)?;
        }
        Err(e) => {
            warn!("Failed to send DM to {}: {}", user.name, e);
            return Err(e.into()); // Convert to BotError
        }
    }

    Ok(())
}
