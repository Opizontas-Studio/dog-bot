use dc_bot::{config::BOT_CONFIG, handler::PingHandler};
use serenity::{Client, all::GatewayIntents};
#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    tracing::info!("Look ma, I'm tracing!");
    // Set gateway intents, which decides what events the bot will be notified about
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;

    // Create a new instance of the Client, logging in as a bot. This will automatically prepend
    // your bot token with "Bot ", which is a requirement by Discord for bot users.
    let mut client = Client::builder(&BOT_CONFIG.token, intents)
        .event_handler(PingHandler)
        .await
        .expect("Err creating client");

    // Finally, start a single shard, and start listening to events.
    //
    // Shards will automatically attempt to reconnect, and will perform exponential backoff until
    // it reconnects.
    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
