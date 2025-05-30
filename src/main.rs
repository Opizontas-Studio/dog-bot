use dc_bot::{
    config::BOT_CONFIG,
    framework::{health, supervisors},
    handler::PingHandler,
};
use serenity::{Client, all::GatewayIntents};
use tracing::error;
use tracing_subscriber::EnvFilter;
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_ansi(true) // Force ANSI colors
        .init();

    tracing::info!("Look ma, I'm tracing!");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&BOT_CONFIG.token, intents)
        .event_handler(PingHandler)
        .framework(supervisors::framework())
        .framework(health::framework())
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
