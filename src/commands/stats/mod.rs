mod active;
mod channel;
mod user;
pub use active::*;
pub use channel::*;
use poise::command;
pub use user::*;

use super::Context;
use crate::{database::DB, error::BotError, services::MessageService};
#[command(slash_command, guild_only, owners_only, ephemeral)]
/// **危险** 清除所有频道统计数据，请在确认表单中输入 "yes" 以确认。
pub async fn nuke_channel_stats(ctx: Context<'_>, confirm: String) -> Result<(), BotError> {
    if confirm != "yes" {
        ctx.reply("请使用正确的确认文本来清除频道统计数据。")
            .await?;
        return Ok(());
    }
    if let Err(why) = DB.message().nuke().await {
        ctx.reply(format!("Failed to nuke channel stats: {why}"))
            .await?;
        return Err(BotError::from(why));
    }
    ctx.reply("频道统计数据已被清除。").await?;
    Ok(())
}
