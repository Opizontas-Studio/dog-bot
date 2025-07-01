use chrono::{FixedOffset, Utc};
use dc_bot::{config::BOT_CONFIG, error::BotError, framework::framework, handler::*};
use serenity::{Client, all::GatewayIntents};
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::Writer, time::FormatTime},
};

struct TimeFormatter;

impl FormatTime for TimeFormatter {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let offset = BOT_CONFIG.load().time_offset;
        let now = Utc::now().with_timezone(
            &FixedOffset::east_opt(offset)
                .expect("Failed to create FixedOffset with the configured time offset"),
        );
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f %Z"))
    }
}

#[tokio::main]
async fn main() -> Result<(), BotError> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .with_timer(TimeFormatter)
        .init();

    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged() | GatewayIntents::privileged();

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&BOT_CONFIG.load().token, intents)
        .event_handler(PingHandler)
        // .event_handler(ClewdrHandler)
        .event_handler(CookieHandler)
        .event_handler(TreeHoleHandler::default())
        .event_handler(FlushHandler)
        .event_handler(ActiveHandler)
        .framework(framework())
        .await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    Ok(client.start().await?)
}
