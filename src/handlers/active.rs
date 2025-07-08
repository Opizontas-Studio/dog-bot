use serenity::all::*;
use tracing::warn;

use crate::database::GetDb;

pub struct ActiveHandler;

#[async_trait]
impl EventHandler for ActiveHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        let Some(guild_id) = msg.guild_id else { return };
        if msg.author.bot || msg.author.system {
            return;
        }
        let message_id = msg.id;
        let channel_id = msg.channel_id;
        let user_id = msg.author.id;
        let timestamp = msg.timestamp;

        if let Err(why) = ctx
            .db()
            .await
            .expect("Failed to get database")
            .message()
            .record(message_id, user_id, guild_id, channel_id, timestamp)
            .await
        {
            warn!("Error recording message: {why:?}");
        }
    }

    // async fn message_delete(
    //     &self,
    //     ctx: Context,
    //     channel_id: ChannelId,
    //     message_id: MessageId,
    //     guild_id: Option<GuildId>,
    // ) {
    //     const CIA: ChannelId = ChannelId::new(1382012639714086912);
    //     let message = if let Some(message) = ctx.to_owned().cache.message(channel_id, message_id) {
    //         Some(message.to_owned())
    //     } else {
    //         None
    //     };
    //     let Some(message) = message else {
    //         return;
    //     };
    //     let message = message.to_owned();
    //     if message.author.bot || message.author.system {
    //         return;
    //     }
    //     let embed = CreateEmbed::default()
    //         .title("Message Deleted")
    //         .colour(colours::roles::DARK_RED)
    //         .author(
    //             CreateEmbedAuthor::new(message.author.name.to_owned())
    //                 .icon_url(message.author.face()),
    //         )
    //         .field("Channel", channel_id.mention().to_string(), true)
    //         .field(
    //             "Guild",
    //             guild_id
    //                 .and_then(|id| id.name(ctx.to_owned()))
    //                 .unwrap_or_else(|| "Unknown".into()),
    //             true,
    //         )
    //         .description(message.content.to_owned())
    //         .footer(CreateEmbedFooter::new(format!(
    //             "Message ID: {}",
    //             message_id
    //         )))
    //         .timestamp(message.timestamp);
    //     if let Err(why) = CIA
    //         .send_message(ctx, CreateMessage::default().embed(embed))
    //         .await
    //     {
    //         warn!("Error sending message delete notification: {why:?}");
    //     }
    // }
}
