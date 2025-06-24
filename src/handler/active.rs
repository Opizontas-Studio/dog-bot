use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::warn;

use crate::database::DB;

pub struct ActiveHandler;

#[async_trait]
impl EventHandler for ActiveHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        let Some(guild_id) = msg.guild_id else { return };
        let user_id = msg.author.id;
        let timestamp = msg.timestamp;

        if let Err(why) = DB.actives().insert(user_id, guild_id, timestamp) {
            warn!("Error inserting active data: {why:?}");
        }
    }
}
