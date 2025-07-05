use serenity::{async_trait, model::channel::Message, prelude::*};
use tracing::warn;

pub struct ClewdrHandler;

#[async_trait]
impl EventHandler for ClewdrHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content.to_lowercase().contains("clewd")
            && !msg.content.to_lowercase().contains("clewdr")
        {
            if let Err(why) = msg.reply(&ctx.http, "请使用 ClewdR 喵~").await {
                warn!("Error sending ClewdR message: {why:?}");
            }
        }
    }
}
