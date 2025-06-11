use super::Context;
use crate::{
    config::{BOT_CONFIG, BotCfg},
    error::BotError,
};
use poise::command;
use serenity::all::*;
use std::time::Duration;

pub mod command {
    use std::collections::HashSet;

    use poise::CreateReply;

    use super::*;
    #[command(
        slash_command,
        guild_only,
        required_permissions = "ADMINISTRATOR",
        name_localized("zh-CN", "注册树洞"),
        description_localized("zh-CN", "添加一个树洞"),
        ephemeral
    )]
    /// Registers a tree hole channel for auto message cleanup.
    pub async fn register_tree_hole(
        ctx: Context<'_>,
        #[name_localized("zh-CN", "树洞频道")]
        #[description_localized("zh-CN", "要注册的树洞频道")]
        #[description = "The tree hole channel to register"]
        channel: Channel,
        #[name_localized("zh-CN", "清理时间")]
        #[description_localized("zh-CN", "清理时间, 单位为秒")]
        #[description = "The cleanup time in seconds"]
        secs: u64,
    ) -> Result<(), BotError> {
        // channel must be a text channel
        if let Some(guild_channel) = channel.to_owned().guild() {
            if guild_channel.guild_id != ctx.guild_id().unwrap_or_default() {
                ctx.say("❌ **错误**\n\n树洞频道必须在当前服务器中。")
                    .await?;
                return Ok(());
            }
            if guild_channel.kind != ChannelType::Text {
                ctx.say("❌ **错误**\n\n树洞频道必须是文本频道。").await?;
                return Ok(());
            }
        } else {
            ctx.say("❌ **错误**\n\n树洞频道必须是服务器频道。").await?;
            return Ok(());
        }
        BOT_CONFIG.rcu(|cfg| {
            let mut cfg = BotCfg::clone(cfg);
            cfg.tree_holes
                .insert(channel.id(), Duration::from_secs(secs));
            cfg
        });
        if let Err(why) = BOT_CONFIG.load().write() {
            ctx.say(format!("❌ **错误**\n\n无法更新配置文件: {why:?}"))
                .await?;
            return Err(why.into());
        }
        ctx.say(format!(
            "✅ **成功**\n\n树洞频道 {} 已注册, 清理时间为 {} 秒。",
            channel.mention(),
            secs
        ))
        .await?;
        Ok(())
    }

    #[command(
        slash_command,
        guild_only,
        required_permissions = "ADMINISTRATOR",
        name_localized("zh-CN", "取消注册树洞"),
        description_localized("zh-CN", "取消注册树洞频道"),
        ephemeral
    )]
    pub async fn unregister_tree_hole(ctx: Context<'_>, channel: Channel) -> Result<(), BotError> {
        if let Some(guild_channel) = channel.to_owned().guild() {
            if guild_channel.guild_id != ctx.guild_id().unwrap_or_default() {
                ctx.say("❌ **错误**\n\n树洞频道必须在当前服务器中。")
                    .await?;
                return Ok(());
            }
            if guild_channel.kind != ChannelType::Text {
                ctx.say("❌ **错误**\n\n树洞频道必须是文本频道。").await?;
                return Ok(());
            }
        } else {
            ctx.say("❌ **错误**\n\n树洞频道必须是服务器频道。").await?;
            return Ok(());
        }
        if !BOT_CONFIG.load().tree_holes.contains_key(&channel.id()) {
            ctx.say("❌ **错误**\n\n该频道不是注册的树洞频道。").await?;
            return Ok(());
        }
        BOT_CONFIG.rcu(|cfg| {
            let mut cfg = BotCfg::clone(cfg);
            cfg.tree_holes.remove(&channel.id());
            cfg
        });
        if let Err(why) = BOT_CONFIG.load().write() {
            ctx.say(format!("❌ **错误**\n\n无法更新配置文件: {why:?}"))
                .await?;
            return Err(why.into());
        }
        ctx.say(format!(
            "✅ **成功**\n\n树洞频道 {} 已取消注册。",
            channel.mention()
        ))
        .await?;
        Ok(())
    }

    #[command(
        slash_command,
        guild_only,
        required_permissions = "ADMINISTRATOR",
        name_localized("zh-CN", "列出树洞"),
        description_localized("zh-CN", "列出当前服务器注册的树洞频道"),
        ephemeral
    )]
    pub async fn list_tree_holes(ctx: Context<'_>) -> Result<(), BotError> {
        let current_channels = ctx
            .guild()
            .unwrap()
            .channels
            .keys()
            .cloned()
            .collect::<HashSet<_>>();
        let holes = BOT_CONFIG
            .load()
            .tree_holes
            .iter()
            .filter(|(channel_id, _)| current_channels.contains(channel_id))
            .map(|(channel_id, duration)| (*channel_id, *duration))
            .collect::<Vec<_>>();

        if holes.is_empty() {
            ctx.say("当前没有注册的树洞频道。").await?;
            return Ok(());
        }

        let reply = CreateReply::default().content("当前注册的树洞频道:").embed(
            CreateEmbed::new()
                .title("树洞频道列表")
                .field("数量", holes.len().to_string(), true)
                .description(
                    holes
                        .iter()
                        .map(|(channel_id, duration)| {
                            format!("- {}: {} 秒", channel_id.mention(), duration.as_secs())
                        })
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .color(0x00FF00),
        );
        ctx.send(reply).await?;
        Ok(())
    }
}
