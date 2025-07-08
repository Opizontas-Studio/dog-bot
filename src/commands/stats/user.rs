use std::time::Instant;

use chrono::{DateTime, Utc};
use futures::{StreamExt, stream};
use poise::{CreateReply, command};
use serenity::all::{colours::roles::DARK_GREEN, *};

use super::super::Context;
use crate::{error::BotError, services::MessageService, utils::get_all_children_channels};

#[command(slash_command, guild_only, owners_only, ephemeral)]
/// 获取用户活跃度统计
pub async fn user_stats(
    ctx: Context<'_>,
    #[description = "显示前 N 个活跃用户，默认为 20"]
    #[min = 1]
    #[max = 50]
    top_n: Option<usize>,
    #[description = "指定服务器 ID, 默认为当前所在服务器"] guild: Option<Guild>,
    #[description = "指定频道, 默认为所有频道"] channel: Option<GuildChannel>,
    #[description = "统计时间范围开始时间, 格式为 RFC3339, 默认无限制"] from: Option<DateTime<Utc>>,
    #[description = "统计时间范围结束时间, 格式为 RFC3339, 默认为现在"] to: Option<DateTime<Utc>>,
    #[description = "是否为临时消息（仅自己可见）"] ephemeral: Option<bool>,
) -> Result<(), BotError> {
    let ephemeral = ephemeral.unwrap_or(true);
    let top_n = top_n.unwrap_or(20); // 默认显示前20个用户
    if ephemeral {
        ctx.defer_ephemeral().await?;
    } else {
        ctx.defer().await?;
    }
    let guild = guild.unwrap_or_else(|| ctx.guild().unwrap().to_owned());
    let guild_id = guild.id;
    let guild_name = guild.name.to_owned();
    let now = Instant::now();
    let channels = channel.as_ref().map(|c| {
        get_all_children_channels(&guild, c)
            .into_iter()
            .map(|c| c.id)
            .collect::<Vec<_>>()
    });
    let db = ctx.data().db.to_owned();
    let data = db
        .message()
        .get_user_stats(guild_id, channels.as_deref(), from, to)
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
    let now = Instant::now();
    let ranking_text = data
        .into_iter()
        .take(top_n)
        .map(async |(user_id, count)| {
            let name = user_id
                .to_user(ctx)
                .await
                .map(|u| u.mention().to_string())
                .unwrap_or_else(|_| user_id.to_string());
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
        .title(format!("{guild_name} 用户活跃度统计"))
        .field("总条数", sum.to_string(), false)
        .field(
            "频道",
            channel
                .map(|c| c.mention().to_string())
                .unwrap_or_else(|| "所有频道".into()),
            false,
        )
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
                from.map_or_else(|| "不限".into(), |f| f.to_rfc3339()),
                to.map_or_else(|| "不限".into(), |t| t.to_rfc3339())
            ),
            false,
        )
        .description(ranking_text)
        .color(DARK_GREEN);
    let reply = CreateReply::default().embed(embed).ephemeral(ephemeral);
    ctx.send(reply).await?;

    Ok(())
}
