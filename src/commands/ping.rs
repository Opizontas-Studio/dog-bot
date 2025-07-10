use chrono::Utc;
use poise::{CreateReply, command};
use serenity::all::CreateEmbed;
use tracing::warn;

use super::Context;
use crate::error::BotError;

#[command(prefix_command)]
/// Ping command to check bot latency
pub async fn ping(ctx: Context<'_>) -> Result<(), BotError> {
    let now = Utc::now();
    let latency = ctx.ping().await;
    let msg_timestamp = ctx.created_at().to_utc();
    let get_latency = now - msg_timestamp;

    let embed = CreateEmbed::new()
        .title("🏓 Pong!")
        .field(
            "WebSocket Latency",
            format!("{} ms", latency.as_millis()),
            true,
        )
        .field(
            "Message Get Latency",
            format!("{} ms", get_latency.num_milliseconds()),
            true,
        )
        .color(0x00FF00)
        .timestamp(now);
    let now = Utc::now();
    let handler = ctx.send(CreateReply::default().embed(embed)).await?;
    let Ok(post_timestamp) = handler.message().await.map(|m| m.timestamp.to_utc()) else {
        warn!("Failed to get message timestamp for ping response");
        return Ok(());
    };
    let post_latency = post_timestamp - now;
    handler
        .edit(
            ctx,
            CreateReply::default().embed(
                CreateEmbed::new()
                    .title("🏓 Pong!")
                    .field(
                        "WebSocket Latency",
                        format!("{} ms", latency.as_millis()),
                        true,
                    )
                    .field(
                        "Message Get Latency",
                        format!("{} ms", get_latency.num_milliseconds()),
                        true,
                    )
                    .field(
                        "Message Post Latency",
                        format!("{} ms", post_latency.num_milliseconds()),
                        true,
                    )
                    .color(0x00FF00)
                    .timestamp(now),
            ),
        )
        .await?;
    Ok(())
}
