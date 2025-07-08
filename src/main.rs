use chrono::{FixedOffset, Utc};
use clap::Parser;
use dc_bot::{
    Args, commands::framework, config::BOT_CONFIG, database::BotDatabase, error::BotError,
    handlers::*,
};
use serenity::{Client, all::GatewayIntents};
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::Writer, time::FormatTime},
};

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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

    let intents = GatewayIntents::non_privileged() | GatewayIntents::privileged();

    let db = BotDatabase::new(&Args::parse().db).await?;
    let mut client = Client::builder(&BOT_CONFIG.load().token, intents)
        .cache_settings({
            let mut s = serenity::cache::Settings::default();
            s.max_messages = 1000; // Set the maximum number of messages to cache
            s
        })
        .type_map_insert::<BotDatabase>(db.to_owned())
        .event_handler(PingHandler)
        .event_handler(CookieHandler)
        .event_handler(TreeHoleHandler::default())
        .event_handler(FlushHandler)
        .event_handler(ActiveHandler)
        .framework(framework(db))
        .await?;

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    Ok(client.start().await?)
}
