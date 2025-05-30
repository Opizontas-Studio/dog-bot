use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::{info, warn};

pub struct PingHandler;

#[async_trait]
impl EventHandler for PingHandler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "!help" => {
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "狗 Bot!\nWritten in Rust using Serenity!")
                    .await
                {
                    warn!("Error sending help message: {why:?}");
                }
            }
            _ => {}
        }
        if msg.content.to_lowercase().contains("clewd")
            && !msg.content.to_lowercase().contains("clewdr")
        {
            let message = "请使用 ClewdR 喵~";
            if let Err(why) = msg.reply(&ctx.http, message).await {
                warn!("Error sending ClewdR message: {why:?}");
            }
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
