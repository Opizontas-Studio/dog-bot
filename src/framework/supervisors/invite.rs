use serde::{Deserialize, Serialize};
use serenity::all::{
    ButtonStyle, ChannelId, ComponentInteraction, CreateButton, CreateEmbedFooter,
    CreateInteractionResponse, CreateInteractionResponseMessage, CreateMessage, GuildId, Member,
    MessageId, UserId,
};
use snafu::{OptionExt, whatever};
use tracing::{error, info, warn};

use crate::{config::BOT_CONFIG, database::DB, error::BotError};

use super::super::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invite {
    pub guild_id: GuildId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
}

async fn handle_accept_supervisor(
    ctx: &serenity::all::Context,
    interaction: &ComponentInteraction,
    user_id: UserId,
    guild_id: GuildId,
) -> Result<(), BotError> {
    let Ok(member) = guild_id.member(ctx, user_id).await else {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("❌ **错误**\n\n抱歉, 我们无法找到您的成员信息。你可能不在这个服务器上。")
                .ephemeral(true),
        );
        interaction.create_response(ctx, response).await?;
        whatever!("Failed to get member information for user {}", user_id);
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
                .content("❌ **错误**\n\n抱歉, 你来晚了！我们现在已经有足够的监督员了。你仍然可以作为志愿者提供帮助！")
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
                .content("❌ **错误**\n\n抱歉, 添加监督员角色时发生错误。请联系管理员。")
                .ephemeral(true),
        );
        interaction.create_response(ctx, response).await?;
    }

    info!("{} accepted supervisor invitation", interaction.user.name);
    let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("✅ **恭喜！**\n\n您现在是监督员了! 欢迎加入团队。如果您想要辞去这个角色, 可以使用 `/resign_supervisor`。")
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
            .content("👍 **没问题！**\n\n您已拒绝监督员邀请。如果将来需要更多监督员, 您可能会收到另一个邀请。")
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
    let Some(invite) = DB.invites().remove(user_id)? else {
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
    if let Err(e) = invite
        .channel_id
        .message(ctx, invite.message_id)
        .await?
        .delete(ctx)
        .await
    {
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
    Ok(())
}

/// Get all members with the volunteer role who aren't already supervisors
async fn get_eligible_volunteers(ctx: Context<'_>) -> Result<Vec<Member>, BotError> {
    let guild = ctx
        .guild()
        .whatever_context::<&str, BotError>("Failed to get guild information")?;
    let pending = DB.invites().pending()?;
    let volunteer_role_id = BOT_CONFIG.volunteer_role_id;
    let supervisor_role_id = BOT_CONFIG.supervisor_role_id;
    Ok(guild
        .members
        .values()
        .filter(|member| {
            member.roles.contains(&volunteer_role_id)
                && !member.roles.contains(&supervisor_role_id)
                && !pending.contains(&member.user.id)
        })
        .cloned()
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
        .label("接受")
        .style(ButtonStyle::Success);

    let decline_button = CreateButton::new("decline_supervisor")
        .label("拒绝")
        .style(ButtonStyle::Danger);

    let message = CreateMessage::new()
        .embed(
            serenity::all::CreateEmbed::new()
                .title("你被邀请成为监督员！")
                .description("我们需要你的帮助来监督社区工作。请点击下面的按钮接受或拒绝邀请。")
                .footer(CreateEmbedFooter::new(
                    "如果你不想成为监督员，可以随时拒绝邀请。",
                ))
                .color(0x00FF00),
        )
        .button(accept_button)
        .button(decline_button);

    match user.direct_message(ctx, message).await {
        Ok(m) => {
            info!("Sent supervisor invitation to {}", user.name);
            // Add to pending invitations
            DB.invites()
                .insert(target_user, guild_id, m.channel_id, m.id)?;
        }
        Err(e) => {
            warn!("Failed to send DM to {}: {}", user.name, e);
            return Err(e.into()); // Convert to BotError
        }
    }

    Ok(())
}
