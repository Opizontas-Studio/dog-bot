use poise::{CreateReply, command};
use serenity::all::CreateEmbed;

use super::super::Context;
use crate::error::BotError;

#[command(prefix_command)]
/// Help command to show bot information
pub async fn help(ctx: Context<'_>) -> Result<(), BotError> {
    let embed = CreateEmbed::new()
        .title("🐕 狗 Bot!")
        .description("Written in Rust using Serenity!")
        .color(0x7289DA)
        .field("Language", "Rust 🦀", true)
        .field("Framework", "Serenity + Poise", true)
        .field("Purpose", "Community management and utilities", true)
        .timestamp(chrono::Utc::now());

    ctx.send(CreateReply::default().embed(embed)).await?;
    Ok(())
}
