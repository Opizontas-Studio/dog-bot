use super::super::Context;
use crate::error::BotError;
use crate::services::MessageService;
use futures::{StreamExt, stream};
use itertools::Itertools;
use poise::{CreateReply, command};
use serenity::all::colours::roles::DARK_GREEN;
use serenity::all::*;
use std::time::Instant;
pub mod command {

    use chrono::{DateTime, Utc};

    use super::*;

    #[command(slash_command, guild_only, owners_only)]
    /// 获取频道活跃度统计
    pub async fn channel_stats(
        ctx: Context<'_>,
        #[description = "显示前 N 个活跃频道，默认为 20"]
        #[max = 30]
        top_n: Option<usize>,
        #[description = "统计时间范围开始时间，默认无限制"] from: Option<DateTime<Utc>>,
        #[description = "统计时间范围结束时间，默认为现在"] to: Option<DateTime<Utc>>,
        #[description = "是否为临时消息（仅自己可见）"] ephemeral: Option<bool>,
    ) -> Result<(), BotError> {
        let ephemeral = ephemeral.unwrap_or(true);
        let top_n = top_n.unwrap_or(20); // 默认显示前20个频道
        if ephemeral {
            ctx.defer_ephemeral().await?;
        } else {
            ctx.defer().await?;
        }
        let guild_id = ctx
            .guild_id()
            .expect("Guild ID should be present in a guild context");
        let now = Instant::now();
        let data = MessageService::get_channel_stats(guild_id, from, to).await?;
        let db_duration = now.elapsed();

        if data.is_empty() {
            ctx.send(
                CreateReply::default()
                    .content("该服务器今天还没有发言记录。")
                    .ephemeral(true),
            )
            .await?;
            return Ok(());
        }
        let sum = data.iter().map(|(_, count)| *count).sum::<u64>();
        let now = Instant::now();
        let ranking_text = data
            .into_iter()
            .sorted_unstable_by_key(|(_, count)| std::cmp::Reverse(*count))
            .take(top_n)
            .map(async |(channel_id, count)| {
                let name = channel_id
                    .to_channel(ctx)
                    .await
                    .ok()
                    .and_then(|c| c.guild())
                    .map(|g| g.name)
                    .unwrap_or_else(|| channel_id.to_string());
                (name, count)
            })
            .collect::<stream::FuturesOrdered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .enumerate()
            .map(|(i, (name, count))| {
                format!(
                    "{}. {} ({:.2}%) - {}",
                    i + 1,
                    count,
                    (count * 100) as f64 / sum as f64,
                    name,
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let network_duration = now.elapsed();
        let embed = CreateEmbed::default()
            .title("频道活跃度统计")
            .field("总条数", sum.to_string(), false)
            .field(
                "数据库查询耗时",
                format!("{}ms", db_duration.as_millis()),
                true,
            )
            .field(
                "网络请求耗时",
                format!("{}ms", network_duration.as_millis()),
                true,
            )
            .description(ranking_text)
            .color(DARK_GREEN);
        let reply = CreateReply::default().embed(embed).ephemeral(ephemeral);
        ctx.send(reply).await?;

        Ok(())
    }

    #[command(slash_command, guild_only, owners_only, ephemeral)]
    /// **危险** 清除所有频道统计数据，请在确认表单中输入 "yes" 以确认。
    pub async fn nuke_channel_stats(ctx: Context<'_>, confirm: String) -> Result<(), BotError> {
        if confirm != "yes" {
            ctx.reply("请使用正确的确认文本来清除频道统计数据。")
                .await?;
            return Ok(());
        }
        if let Err(why) = MessageService::nuke_all_messages().await {
            ctx.reply(format!("Failed to nuke channel stats: {why}"))
                .await?;
            return Err(BotError::from(why));
        }
        ctx.reply("频道统计数据已被清除。").await?;
        Ok(())
    }
}
