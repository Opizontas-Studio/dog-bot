use owo_colors::OwoColorize as _;
use serenity::{
    all::{GuildId, Ready, ResumedEvent},
    async_trait,
    prelude::*,
};
use tracing::info;

pub struct BootHandler;

#[async_trait]
impl EventHandler for BootHandler {
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

    async fn resume(&self, _ctx: Context, _resumed: ResumedEvent) {
        // This is called when the bot has resumed a session.
        // You can use this to log that the bot has resumed.
        info!("Bot has resumed successfully.");
    }

    async fn ready(&self, _ctx: Context, ready: Ready) {
        // This is called when the bot is ready and has connected to Discord.
        // You can use this to set the bot's activity or status.
        info!("{} is connected!", ready.user.name.green());
    }
}
