use poise::command;
use serenity::all::Member;
use snafu::OptionExt;
use tracing::{info, warn};

use crate::{config::BOT_CONFIG, error::BotError};

use super::invite::send_supervisor_invitation;

use super::Context;

#[command(prefix_command, guild_only, owners_only)]
pub async fn test_add_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    member.add_role(ctx, BOT_CONFIG.supervisor_role_id).await?;
    info!("{} has been added as a supervisor", ctx.author().name);
    ctx.say("You have been added as a supervisor!").await?;
    Ok(())
}

#[command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), BotError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

/// Quits the current user from being a supervisor and potentially invites a new one.
#[command(slash_command, guild_only, owners_only)]
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
#[command(slash_command, guild_only, owners_only)]
pub async fn invite_supervisor(ctx: Context<'_>, member: Member) -> Result<(), BotError> {
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
