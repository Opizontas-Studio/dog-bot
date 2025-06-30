use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::warn;

use crate::database::DB;
use crate::services::MessageService;

pub struct ActiveHandler;

#[async_trait]
impl EventHandler for ActiveHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        let Some(guild_id) = msg.guild_id else { return };
        if msg.author.bot || msg.author.system {
            return;
        }
        let message_id = msg.id;
        let channel_id = msg.channel_id;
        let user_id = msg.author.id;
        let timestamp = msg.timestamp;

        if let Err(why) = DB
            .message()
            .record(message_id, user_id, guild_id, channel_id, timestamp)
            .await
        {
            warn!("Error recording message: {why:?}");
        }
    }
}
