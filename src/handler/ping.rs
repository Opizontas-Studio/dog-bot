use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use sysinfo::System;
use tracing::warn;

pub struct PingHandler;

#[async_trait]
impl EventHandler for PingHandler {
    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        match msg.content.as_str() {
            "!ping" => {
                // Sending a message can fail, due to a network error, an authentication error, or lack
                // of permissions to post in the channel, so log to stdout when some error happens,
                // with a description of it.
                if let Err(why) = msg
                    .channel_id
                    .say(&ctx.http, "Pong!\nPowered by Serenity in Rust!")
                    .await
                {
                    warn!("Error sending message: {why:?}");
                }
            }
            "!health" => {
                let mut sys = System::new_all();
                sys.refresh_all();
                let cpu_usage = sys.global_cpu_usage();
                let total_memory = sys.total_memory();
                let used_memory = sys.used_memory();
                let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
                let message = format!(
                    "CPU Usage: {:.2}%\nMemory Usage: {:.2}%",
                    cpu_usage, memory_usage
                );
                if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                    warn!("Error sending health message: {why:?}");
                }
            }
            "!sysinfo" => {
                let sys_name = System::name().unwrap_or("Unknown".into());
                let kernel_version = System::kernel_version().unwrap_or("Unknown".into());
                let os_version = System::os_version().unwrap_or("Unknown".into());
                let message = format!(
                    "System Name: {}\nKernel Version: {}\nOS Version: {}",
                    sys_name, kernel_version, os_version
                );
                if let Err(why) = msg.channel_id.say(&ctx.http, message).await {
                    warn!("Error sending sysinfo message: {why:?}");
                }
            }
            _ => {}
        }
    }

    // Set a handler to be called on the `ready` event. This is called when a shard is booted, and
    // a READY payload is sent by Discord. This payload contains data like the current user's guild
    // Ids, current user data, private channels, and more.
    //
    // In this case, just print what the current user's username is.
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}
