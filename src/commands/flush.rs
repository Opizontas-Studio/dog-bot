use std::collections::HashSet;

use chrono::Duration;
use itertools::Itertools;
use poise::{CreateReply, command};
use serenity::all::*;

use crate::{
    commands::{Context, check_admin},
    config::BOT_CONFIG,
    database::DB,
    error::BotError,
    services::FlushService,
};

pub const FLUSH_EMOJI: &str = "⚠️";
pub const DURATION: Duration = Duration::hours(1);

#[command(
    context_menu_command = "冲水",
    guild_only,
    name_localized("zh-CN", "冲水"),
    description_localized("zh-CN", "冲掉一条消息"),
    channel_cooldown = 300
)]
/// Flush a message.
pub async fn flush_message(ctx: Context<'_>, message: Message) -> Result<(), BotError> {
    if BOT_CONFIG
        .load()
        .supervisor_guilds
        .contains(&ctx.guild_id().unwrap_or_default())
        && !check_admin(ctx.to_owned()).await?
    {
        ctx.say("❌ You do not have permission to flush messages in this guild.")
            .await?;
        return Ok(());
    }
    // check if message and ctx is in same channel
    if message.channel_id != ctx.channel_id() {
        ctx.say("❌ The message is not in the same channel as the command.")
            .await?;
        return Ok(());
    }
    if message.pinned {
        ctx.say("❌ You cannot flush a pinned message.").await?;
        return Ok(());
    }
    // check if guild has a toilet
    let guild_channels = ctx
        .guild()
        .unwrap()
        .channels
        .keys()
        .cloned()
        .collect::<HashSet<_>>();
    // intersect with toilets
    let toilet = guild_channels
        .intersection(&BOT_CONFIG.load().toilets)
        .next()
        .cloned();
    let Some(toilet) = toilet else {
        ctx.say("❌ This guild does not have a toilet configured.")
            .await?;
        return Ok(());
    };
    if DB.flush().has(&message).await? {
        ctx.say("❌ This message has already been flushed.").await?;
        return Ok(());
    }
    let threshold = ctx
        .guild_channel()
        .await
        .unwrap()
        .messages(ctx.to_owned(), GetMessages::new())
        .await?
        .into_iter()
        .map(|m| m.author.id)
        .unique()
        .count()
        .div_ceil(2)
        .max(2); // minimum threshold is 2
    let reason = None; // TODO: allow user to provide a reason
    let reply = CreateReply::default()
        .embed(
            CreateEmbed::new()
                .title("冲水投票已创建")
                .thumbnail(message.author.avatar_url().unwrap_or_default())
                .color(0xFF0000)
                .field("消息", message.link(), false)
                .field("消息作者", message.author.mention().to_string(), true)
                .field("冲水发起人", ctx.author().mention().to_string(), true)
                .field("投票阈值", threshold.to_string(), true)
                .field(
                    "冲水理由",
                    reason.to_owned().unwrap_or_else(|| "无".into()),
                    false,
                )
                .description(
                    "请在 1 小时内，使用 ⚠️ 对原始消息或者该消息进行投票，超过阈值则会被冲掉。",
                ),
        )
        .ephemeral(false);
    let ntf = ctx.send(reply).await?;
    let ntf_msg = ntf.into_message().await?;
    DB.flush()
        .insert(
            &message,
            &ntf_msg,
            ctx.author().id,
            toilet,
            threshold as u64,
            reason,
        )
        .await?;
    Ok(())
}
