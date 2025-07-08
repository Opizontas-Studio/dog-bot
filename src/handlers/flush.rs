use chrono::Utc;
use itertools::Itertools;
use serenity::all::*;
use tracing::{error, info, warn};

use crate::{
    commands::flush::{DURATION, FLUSH_EMOJI},
    database::GetDb,
    error::BotError,
};

pub struct FlushHandler;

#[async_trait]
impl EventHandler for FlushHandler {
    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        // delete flush older than 1 hour
        if let Err(e) = ctx
            .db()
            .await
            .expect("Failed to get database")
            .flush()
            .clean(DURATION)
            .await
        {
            error!("Failed to clean flushes: {e}");
        } else {
            info!("Successfully cleaned flushes older than 1 hour.");
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let f = async move || -> Result<(), BotError> {
            if !reaction.emoji.unicode_eq(FLUSH_EMOJI) {
                return Ok(()); // Not a flush reaction, ignore
            }
            let db = ctx.db().await.expect("Failed to get database");
            let Some(flush_info) = db.flush().get(reaction.message_id).await? else {
                return Ok(());
            };
            let msg = ctx
                .http
                .get_message(flush_info.channel_id(), flush_info.message_id())
                .await?;
            let reaction_type = reaction.emoji.to_owned();
            if msg
                .timestamp
                .checked_add_signed(DURATION)
                .is_some_and(|t| t < Utc::now())
            {
                warn!("Flush reaction on a message older than 1 hour, ignoring.");
                db.flush().remove(reaction.message_id).await?;
                return Ok(());
            }
            let msg_reactions = ctx
                .http
                .get_reaction_users(
                    flush_info.channel_id(),
                    flush_info.message_id(),
                    &reaction_type,
                    100,
                    None,
                )
                .await?;
            let ntf_reactions = ctx
                .http
                .get_reaction_users(
                    flush_info.channel_id(),
                    flush_info.notification_id(),
                    &reaction_type,
                    100,
                    None,
                )
                .await?;
            if msg_reactions
                .iter()
                .chain(ntf_reactions.iter())
                .map(|u| u.id)
                .unique()
                .count()
                < flush_info.threshold() as usize
            {
                return Ok(()); // Not enough reactions, ignore
            }
            // forward the message to the toilet channel
            if let MessageType::Regular | MessageType::InlineReply = msg.kind {
                let mut reference = MessageReference::from(&msg);
                reference.kind = MessageReferenceKind::Forward;
                flush_info
                    .toilet_id()
                    .send_message(
                        ctx.to_owned(),
                        CreateMessage::new().reference_message(reference),
                    )
                    .await?;
            }
            // successfully flushed, send to toilet
            let new_msg = CreateMessage::new().add_embed(
                CreateEmbed::new()
                    .title("冲水归档")
                    .color(0xFF0000)
                    .thumbnail(msg.author.avatar_url().unwrap_or_default())
                    .field("消息", msg.link(), false)
                    .field("消息作者", msg.author.mention().to_string(), true)
                    .field(
                        "冲水发起人",
                        flush_info.flusher_id().mention().to_string(),
                        true,
                    )
                    .field(
                        "原因",
                        flush_info.reason.to_owned().unwrap_or_else(|| "无".into()),
                        true,
                    )
                    .field("投票阈值", flush_info.threshold().to_string(), true)
                    .description("该消息已被冲掉。请注意，冲水操作是不可逆的。"),
            );
            flush_info
                .toilet_id()
                .send_message(ctx.to_owned(), new_msg)
                .await?;
            // delete the original message
            ctx.http
                .delete_message(
                    flush_info.channel_id(),
                    flush_info.message_id(),
                    flush_info.reason.as_deref(),
                )
                .await?;

            let delete_msg = CreateMessage::new()
                .add_embed(
                    CreateEmbed::new()
                        .title("冲水成功")
                        .description(format!(
                            "消息 {} 已被 {} 冲掉。",
                            msg.id,
                            flush_info.flusher_id().mention()
                        ))
                        .color(0x00FF00),
                )
                .reference_message((flush_info.channel_id(), flush_info.notification_id()));
            // send a confirmation message to the channel
            reaction
                .channel_id
                .send_message(ctx.to_owned(), delete_msg)
                .await?;

            info!(
                "Successfully flushed message {} by {}",
                msg.id,
                flush_info.flusher_id().mention()
            );

            // remove the flush info from the database
            db.flush().remove(reaction.message_id).await?;

            Ok(())
        };
        if let Err(e) = f().await {
            error!("Error handling flush reaction: {}", e);
        }
    }
}
