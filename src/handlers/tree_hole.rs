use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use chrono::TimeDelta;
use futures::{StreamExt, TryStreamExt, stream::FuturesUnordered};
use moka::{Expiry, notification::RemovalCause, sync::Cache};
use serenity::{all::*, json::json};
use tokio::spawn;
use tracing::error;

use crate::{config::GetCfg, error::BotError};

const MAX_CACHE_CAPACITY: u64 = 10000;
const BATCH_DELETE_SIZE: usize = 100;
const DELETE_REASON: &str = "树洞";

pub struct TreeHoleHandler {
    moka: Cache<MessageId, (Duration, ChannelId, Arc<Http>)>,
}

struct MyExpiry;

impl Expiry<MessageId, (Duration, ChannelId, Arc<Http>)> for MyExpiry {
    fn expire_after_create(
        &self,
        _key: &MessageId,
        value: &(Duration, ChannelId, Arc<Http>),
        _now: Instant,
    ) -> Option<Duration> {
        Some(value.0)
    }
}

fn eviction_listener(
    msg_id: Arc<MessageId>,
    (_, channel_id, http): (Duration, ChannelId, Arc<Http>),
    cause: RemovalCause,
) {
    spawn(async move {
        if cause == RemovalCause::Expired
            && let Ok(msg) = http.get_message(channel_id, *msg_id).await
            && !msg.pinned
            && let Err(err) = msg.delete(http).await
        {
            error!("Failed to delete message {msg_id}: {err}");
        }
    });
}

impl Default for TreeHoleHandler {
    fn default() -> Self {
        Self {
            moka: Cache::builder()
                .max_capacity(MAX_CACHE_CAPACITY)
                .expire_after(MyExpiry)
                .eviction_listener(eviction_listener)
                .build(),
        }
    }
}

#[async_trait]
impl EventHandler for TreeHoleHandler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        self.delete_messages(ctx).await;
    }

    async fn channel_pins_update(&self, ctx: Context, event: ChannelPinsUpdateEvent) {
        if !ctx
            .cfg()
            .await
            .expect("Failed to get bot configuration")
            .load()
            .tree_holes
            .contains_key(&event.channel_id)
        {
            return; // Not a tree hole channel, ignore the message
        };
        // Maybe some previously pinned messages were unpinned, so we need to check again
        self.delete_messages(ctx).await;
    }

    // Set a handler for the `message` event. This is called whenever a new message is received.
    //
    // Event handlers are dispatched through a threadpool, and so multiple events can be
    // dispatched simultaneously.
    async fn message(&self, ctx: Context, msg: Message) {
        let channel_id = msg.channel_id;
        let Some(dur) = ctx
            .cfg()
            .await
            .expect("Failed to get bot configuration")
            .load()
            .tree_holes
            .get(&channel_id)
            .cloned()
        else {
            return; // Not a tree hole channel, ignore the message
        };
        // Store the handle in the map
        self.moka
            .entry(msg.id)
            .or_insert_with(|| (dur, channel_id, ctx.http.to_owned()));
    }

    async fn resume(&self, ctx: Context, _resumed: ResumedEvent) {
        self.delete_messages(ctx).await;
    }
}

impl TreeHoleHandler {
    async fn delete_in_channel(
        &self,
        ctx: Context,
        channel_id: ChannelId,
        dur: Duration,
    ) -> Result<(), BotError> {
        let messages = channel_id
            .messages_iter(ctx.to_owned())
            .try_collect::<Vec<_>>()
            .await?;

        let delta = TimeDelta::from_std(dur).unwrap();
        let now = chrono::Utc::now();

        let filtered = messages
            .into_iter()
            .filter(|msg| !msg.pinned && !self.moka.contains_key(&msg.id));

        let (wait, immediate): (Vec<_>, Vec<_>) = filtered
            .map(|msg| {
                let new_dur = delta - (now - msg.timestamp.to_utc());
                (msg, new_dur)
            })
            .partition(|(_, new_dur)| *new_dur > chrono::Duration::zero());

        wait.into_iter().for_each(|(msg, new_dur)| {
            self.moka.entry(msg.id).or_insert_with(|| {
                (
                    new_dur.to_std().unwrap_or_default(),
                    channel_id,
                    ctx.http.to_owned(),
                )
            });
        });

        let immediate_ids: Vec<_> = immediate.into_iter().map(|(msg, _)| msg.id).collect();

        immediate_ids
            .chunks(BATCH_DELETE_SIZE)
            .map(async |chunk| {
                if let [m] = chunk {
                    // If there's only one message, we must use the simpler delete_message method
                    ctx.http
                        .delete_message(channel_id, *m, Some(DELETE_REASON))
                        .await?
                } else {
                    ctx.http
                        .delete_messages(
                            channel_id,
                            &json!({"messages": chunk}),
                            Some(DELETE_REASON),
                        )
                        .await?
                };
                Ok(())
            })
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .filter_map(Result::err)
            .for_each(|e: serenity::Error| {
                error!("Failed to delete old messages in channel {channel_id}: {e}");
            });
        Ok(())
    }

    async fn delete_messages(&self, ctx: Context) {
        for (channel_id, dur) in ctx
            .cfg()
            .await
            .expect("Failed to get bot configuration")
            .load()
            .tree_holes
            .iter()
        {
            if let Err(e) = self
                .delete_in_channel(ctx.to_owned(), *channel_id, *dur)
                .await
            {
                error!("Failed to delete messages in tree hole channel {channel_id}: {e}");
            }
        }
    }
}
