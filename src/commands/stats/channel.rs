use std::time::Instant;

use chrono::{DateTime, FixedOffset, SecondsFormat};
use futures::{StreamExt, stream};
use poise::{CreateReply, command};
use serenity::all::{colours::roles::DARK_GREEN, *};

use super::{
    super::{Context, check_admin},
    guild_choices, timestamp_choices,
};
use crate::error::BotError;

/// 获取频道活跃度统计
#[command(slash_command, guild_only, ephemeral, check = "check_admin")]
pub async fn channel_stats(
    ctx: Context<'_>,
    #[description = "显示前 N 个活跃频道，默认为 20"]
    #[min = 1]
    #[max = 50]
    top_n: Option<usize>,
    #[description = "指定服务器 ID, 默认为当前服务器"]
    #[autocomplete = "guild_choices"]
    guild: Option<Guild>,
    #[description = "统计时间范围开始时间, 格式为 RFC3339, 默认无限制"]
    #[autocomplete = "timestamp_choices"]
    from: Option<DateTime<FixedOffset>>,
    #[description = "统计时间范围结束时间, 格式为 RFC3339, 默认为现在"]
    #[autocomplete = "timestamp_choices"]
    to: Option<DateTime<FixedOffset>>,
    #[description = "是否为临时消息（仅自己可见）"] ephemeral: Option<bool>,
) -> Result<(), BotError> {
    let ephemeral = ephemeral.unwrap_or(true);
    let top_n = top_n.unwrap_or(20); // 默认显示前20个频道
    if ephemeral {
        ctx.defer_ephemeral().await?;
    } else {
        ctx.defer().await?;
    }
    let guild_id = guild
        .map(|g| g.id)
        .or_else(|| ctx.guild_id())
        .expect("Guild ID should be present in a guild context");
    let guild_name = guild_id.name(ctx).unwrap_or_else(|| guild_id.to_string());
    let now = Instant::now();
    let data = ctx
        .data()
        .db
        .message()
        .get_channel_stats(guild_id, from, to)
        .await?;
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
    let sum_f64 = sum as f64;
    let now = Instant::now();
    let ranking_text = data
        .into_iter()
        .take(top_n)
        .map(async |(channel_id, count)| {
            let name = ctx
                .cache()
                .guild(guild_id)
                .and_then(|g| g.channels.get(&channel_id).cloned())
                .map(|c| c.name);
            if let Some(name) = name {
                (name, count)
            } else {
                let channel = channel_id
                    .name(ctx)
                    .await
                    .unwrap_or_else(|_| channel_id.to_string());
                (channel, count)
            }
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
                (count * 100) as f64 / sum_f64,
                name,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let network_duration = now.elapsed();
    let embed = CreateEmbed::default()
        .title(format!("{guild_name} 频道活跃度统计"))
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
        .field(
            "统计时间范围",
            format!(
                "{} - {}",
                from.map_or_else(
                    || "不限".into(),
                    |f| f.to_rfc3339_opts(SecondsFormat::AutoSi, true)
                ),
                to.map_or_else(
                    || "不限".into(),
                    |t| t.to_rfc3339_opts(SecondsFormat::AutoSi, true)
                )
            ),
            false,
        )
        .description(ranking_text)
        .color(DARK_GREEN);
    let reply = CreateReply::default().embed(embed).ephemeral(ephemeral);
    ctx.send(reply).await?;

    Ok(())
}
