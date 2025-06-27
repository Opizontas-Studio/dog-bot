use poise::{CreateReply, command};
use serenity::all::{CreateEmbed, CreateEmbedFooter, Member, Mention};
use snafu::OptionExt;
use tracing::{info, warn};

use crate::{config::BOT_CONFIG, error::BotError};

use super::{super::check_admin, invite::send_supervisor_invitation};

use super::super::Context;

async fn check_guild(ctx: Context<'_>) -> Result<bool, BotError> {
    if !BOT_CONFIG
        .load()
        .supervisor_guilds
        .contains(&ctx.guild_id().unwrap_or_default())
    {
        warn!(
            "Command used in non-supervisor guild: {}",
            ctx.guild_id().unwrap_or_default()
        );
        let reply = CreateReply::default()
            .content("❌ This command can only be used in supervisor guilds.")
            .ephemeral(true);
        ctx.send(reply).await?;
        return Ok(false);
    }
    Ok(true)
}

async fn check_supervisor(ctx: Context<'_>) -> Result<bool, BotError> {
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    if !member.roles.contains(&BOT_CONFIG.load().supervisor_role_id) {
        warn!("{} is not a supervisor", ctx.author().name);
        ctx.say("❌ You are not a supervisor!").await?;
        return Ok(false);
    }
    Ok(true)
}

/// Quits the current user from being a supervisor and potentially invites a new one.
#[command(
    slash_command,
    guild_only,
    check = "check_guild",
    check = "check_supervisor",
    ephemeral
)]
pub async fn resign_supervisor(ctx: Context<'_>) -> Result<(), BotError> {
    // Remove role from member
    let member = ctx
        .author_member()
        .await
        .whatever_context::<&str, BotError>("Failed to get member information")?;
    let role_id = BOT_CONFIG.load().supervisor_role_id;
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
    check = "check_guild",
    check = "check_admin",
    ephemeral
)]
pub async fn invite_supervisor(ctx: Context<'_>, member: Member) -> Result<(), BotError> {
    let volunteer_id = member.user.id;
    let volunteer_name = &member.user.name;
    if member.roles.contains(&BOT_CONFIG.load().supervisor_role_id) {
        ctx.say(format!("❌ **{volunteer_name}** 已经是监督员了！"))
            .await?;
        return Ok(());
    }

    if let Err(e) = send_supervisor_invitation(ctx, volunteer_id).await {
        warn!("Failed to send invitation: {}", e);
        ctx.say(format!(
            "❌ 无法向 **{volunteer_name}** 发送邀请。请检查他们的私信设置。"
        ))
        .await?;
        return Err(e);
    }
    info!("Invited {} to become a supervisor", volunteer_name);
    ctx.say(format!(
        "✅ 已邀请 **{volunteer_name}** 成为监督员。请等待他们的响应。",
    ))
    .await?;

    Ok(())
}

#[command(
    slash_command,
    guild_only,
    check = "check_guild",
    check = "check_admin",
    ephemeral
)]
/// Fetches all supervisors.
pub async fn current_supervisors(ctx: Context<'_>) -> Result<(), BotError> {
    let msg = ctx.say("正在获取当前监督员列表...").await?;
    let members = fetch_all_supervisors(ctx)?;
    if members.is_empty() {
        msg.edit(
            ctx,
            CreateReply::default().embed(
                CreateEmbed::new().description(
                    "当前没有监督员。请使用 `/invite_supervisor` 命令邀请新的监督员。",
                ),
            ),
        )
        .await?;
        return Ok(());
    }
    msg.edit(
        ctx,
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("当前监督员列表")
                .color(0x00FF00)
                .thumbnail(ctx.author().avatar_url().unwrap_or_default())
                .field("数量", members.len().to_string(), true)
                .description(
                    members
                        .iter()
                        .map(|m| format!("{} ({})", m.user.name, Mention::from(m.user.id)))
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .footer(CreateEmbedFooter::new(
                    "如果需要邀请新的监督员，请使用 `/invite_supervisor` 命令。",
                )),
        ),
    )
    .await?;

    Ok(())
}

/// Fetches all members with the supervisor role in the current guild.
pub fn fetch_all_supervisors(ctx: Context<'_>) -> Result<Vec<Member>, BotError> {
    let guild = ctx
        .guild()
        .whatever_context::<&str, BotError>("Failed to get guild information")?
        .to_owned();
    let role_id = BOT_CONFIG.load().supervisor_role_id;
    let members = guild
        .members
        .values()
        .filter(|member| member.roles.contains(&role_id))
        .cloned()
        .collect();

    Ok(members)
}
