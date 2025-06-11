use chrono::Utc;
use chrono_tz::Australia::Sydney;
use dc_bot::{config::BOT_CONFIG, framework::framework, handler::*};
use serenity::{Client, all::GatewayIntents};
use tracing::error;
use tracing_subscriber::{
    EnvFilter,
    fmt::{format::Writer, time::FormatTime},
};

struct AustralianEasternTime;

impl FormatTime for AustralianEasternTime {
    fn format_time(&self, w: &mut Writer<'_>) -> std::fmt::Result {
        let now = Utc::now().with_timezone(&Sydney);
        write!(w, "{}", now.format("%Y-%m-%d %H:%M:%S%.3f %Z"))
    }
}
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true)
        .with_timer(AustralianEasternTime)
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
        .framework(framework())
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        error!("Client error: {why:?}");
    }
}
