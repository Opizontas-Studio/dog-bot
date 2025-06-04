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
#[command(slash_command, guild_only, owners_only, check = "check_guild")]
pub async fn resign_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    let role_id = BOT_CONFIG.supervisor_role_id;

    ctx.defer_ephemeral().await?;
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
#[command(slash_command, guild_only, owners_only, check = "check_guild")]
pub async fn invite_supervisor(ctx: Context<'_>, member: Member) -> Result<(), BotError> {
    let volunteer_id = member.user.id;
    let volunteer_name = &member.user.name;

    ctx.defer_ephemeral().await?;
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
