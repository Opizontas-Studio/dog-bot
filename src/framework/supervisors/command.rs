use poise::command;
use serenity::all::Member;
use snafu::OptionExt;
use tracing::{info, warn};

use crate::{config::BOT_CONFIG, error::BotError};

use super::invite::send_supervisor_invitation;

use super::super::Context;

async fn check_guild(ctx: Context<'_>) -> Result<bool, BotError> {
    if !BOT_CONFIG
        .supervisor_guilds
        .contains(&ctx.guild_id().unwrap_or_default().get())
    {
        warn!(
            "Command used in non-supervisor guild: {}",
            ctx.guild_id().unwrap_or_default()
        );
        ctx.defer_ephemeral().await?;
        ctx.say("❌ This command can only be used in designated supervisor guilds.")
            .await?;
        return Ok(false);
    }
    Ok(true)
}

/// Quits the current user from being a supervisor and potentially invites a new one.
#[command(
    slash_command,
    guild_only,
    owners_only,
    check = "check_guild",
    ephemeral
)]
pub async fn resign_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
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

/// Manually invite a volunteer to become supervisor (for testing/admin use)
#[command(
    slash_command,
    guild_only,
    default_member_permissions = "ADMINISTRATOR",
    check = "check_guild",
    ephemeral
)]
pub async fn invite_supervisor(ctx: Context<'_>, member: Member) -> Result<(), BotError> {
    let volunteer_id = member.user.id;
    let volunteer_name = &member.user.name;

    if let Err(e) = send_supervisor_invitation(ctx, volunteer_id).await {
        warn!("Failed to send invitation: {}", e);
        ctx.say(format!(
            "❌ 无法向 **{}** 发送邀请。请检查他们的私信设置。",
            volunteer_name
        ))
        .await?;
        return Err(e);
    }
    info!("Invited {} to become a supervisor", volunteer_name);
    ctx.say(format!(
        "✅ 已邀请 **{}** 成为监督员。请等待他们的响应。",
        volunteer_name
    ))
    .await?;

    Ok(())
}
