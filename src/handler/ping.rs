use chrono::Utc;
use owo_colors::OwoColorize;
use serenity::{
    all::{EditMessage, GuildId, Ready},
    async_trait,
    model::channel::Message,
    prelude::*,
};
use tracing::{info, warn};

use crate::error::BotError;

pub struct PingHandler;

#[async_trait]
impl EventHandler for PingHandler {
    async fn cache_ready(&self, ctx: Context, guilds: Vec<GuildId>) {
        // This is called when the cache is ready.
        // list all guilds the bot is in
        info!(
            "Cache is ready! Bot is in {} guilds.",
            guilds.len().to_string().green()
        );
        for guild in guilds {
            let guild_name = ctx
                .cache
                .guild(guild)
                .map(|g| g.name.to_owned())
                .unwrap_or("Uncached Guild".to_string());
            info!("Connected to: {} ({})", guild_name.green(), guild);
        }
    }

    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let f = async move || -> Result<(), BotError> {
            match msg.content.as_str() {
                "!ping" => {
                    let now = Utc::now();
                    let msg_time = msg.timestamp.to_utc();
                    let delta_one = now - msg_time;
                    let reply = format!(
                        "Pong!\nReceive Latency: {} ms",
                        delta_one.num_milliseconds()
                    );
                    let mut msg = msg.reply(&ctx.http, reply).await?;
                    let reply_time = msg.timestamp.to_utc();
                    let delta_two = reply_time - msg_time;
                    msg.edit(
                        &ctx.http,
                        EditMessage::new().content(format!(
                            "Pong!\nReceive Latency: {} ms\nReply Latency: {} ms",
                            delta_one.num_milliseconds(),
                            delta_two.num_milliseconds()
                        )),
                    )
                    .await?;
                }
                "!help" => {
                    msg.channel_id
                        .say(&ctx.http, "狗 Bot!\nWritten in Rust using Serenity!")
                        .await?;
                }
                _ => {}
            }
            Ok(())
        };
        if let Err(e) = f().await {
            warn!("Error handling message: {}", e);
        }
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        // This is called when the bot is ready and has connected to Discord.
        // You can use this to set the bot's activity or status.
        info!("{} is connected!", ready.user.name.green());
    }
}
