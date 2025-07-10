mod channel;
mod user;
pub use channel::*;
use poise::command;
use serenity::all::*;
pub use user::*;

use super::Context;
use crate::error::BotError;
#[command(slash_command, guild_only, owners_only, ephemeral)]
/// **危险** 清除所有频道统计数据，请在确认表单中输入 "yes" 以确认。
pub async fn nuke_channel_stats(ctx: Context<'_>, confirm: String) -> Result<(), BotError> {
    if confirm != "yes" {
        ctx.reply("请使用正确的确认文本来清除频道统计数据。")
            .await?;
        return Ok(());
    }
    if let Err(why) = ctx.data().db.message().nuke().await {
        ctx.reply(format!("Failed to nuke channel stats: {why}"))
            .await?;
        return Err(why);
    }
    ctx.reply("频道统计数据已被清除。").await?;
    Ok(())
}

pub async fn timestamp_choices<'a>(
    _ctx: Context<'_>,
    _partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice> + 'a {
    // 1 day ago and 1 week ago
    let now = chrono::Utc::now();
    let one_day_ago = now - chrono::Duration::days(1);
    let one_week_ago = now - chrono::Duration::weeks(1);
    [("1 day ago", one_day_ago), ("1 week ago", one_week_ago)]
        .into_iter()
        .map(|(name, timestamp)| AutocompleteChoice::new(name.to_string(), timestamp.to_rfc3339()))
}

pub async fn guild_choices<'a>(
    ctx: Context<'_>,
    _partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice> + 'a {
    ctx.data()
        .cfg
        .load()
        .monitor_guilds
        .iter()
        .filter_map(|guild| {
            let name = guild.name(ctx)?;
            Some(AutocompleteChoice::new(name, guild.to_string()))
        })
        .collect::<Vec<_>>()
        .into_iter()
}
