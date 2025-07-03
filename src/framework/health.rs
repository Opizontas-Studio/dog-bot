use futures::{StreamExt, stream::FuturesOrdered};
use poise::{CreateReply, command};
use serenity::all::{
    colours::branding::{GREEN, RED, YELLOW},
    *,
};
use snafu::whatever;
use sysinfo::System;
use tracing::error;

use super::Context;
use crate::{database::DB, error::BotError};

#[command(
    slash_command,
    owners_only,
    default_member_permissions = "ADMINISTRATOR",
    subcommands("status", "journal")
)]
pub async fn systemd(_: Context<'_>) -> Result<(), BotError> {
    Ok(())
}

#[command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR",
    owners_only,
    name_localized("zh-CN", "状态"),
    description_localized("zh-CN", "获取 dc-bot.service 的 systemd 状态"),
    ephemeral
)]
/// Fetches the systemd status of the `dc-bot.service`.
async fn status(ctx: Context<'_>) -> Result<(), BotError> {
    // call systemctl status command
    use std::process::Command;
    let output = Command::new("systemctl")
        .arg("status")
        .arg("dc-bot.service")
        .arg("--lines=0")
        .output()?;
    if !output.status.success() {
        error!(
            "Failed to get systemd status: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        whatever!("Failed to get systemd status");
    }
    let status = String::from_utf8_lossy(&output.stdout);
    ctx.say(format!("```ansi\n{}\n```", status.trim())).await?;
    Ok(())
}

#[command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR",
    owners_only,
    name_localized("zh-CN", "日志"),
    description_localized("zh-CN", "获取 dc-bot.service 的 systemd 日志"),
    ephemeral
)]
/// Fetches the systemd journal of the `dc-bot.service`.
async fn journal(
    ctx: Context<'_>,
    #[min = 1]
    #[max = 20]
    #[description = "Number of lines to fetch from the journal"]
    #[name_localized("zh-CN", "行数")]
    #[description_localized("zh-CN", "从日志中获取的行数")]
    lines: Option<usize>,
) -> Result<(), BotError> {
    // call systemctl status command
    use std::process::Command;
    let output = Command::new("journalctl")
        .arg("-u")
        .arg("dc-bot.service")
        .arg("--output=cat")
        .arg(format!("--lines={}", lines.unwrap_or(10)))
        .output()?;
    if !output.status.success() {
        error!(
            "Failed to get systemd journal: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        whatever!("Failed to get systemd journal");
    }
    let status = String::from_utf8_lossy(&output.stdout);
    let output = format!("```ansi\n{}\n```", status.trim());
    // handle message too long
    if let Err(serenity::Error::Model(ModelError::MessageTooLong(_))) = ctx.say(output).await {
        ctx.say("The output is too long to display. Please try a smaller limit.")
            .await?;
        return Ok(());
    }
    Ok(())
}

use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, Statement};
async fn get_db_size(db: &DatabaseConnection) -> Result<i64, DbErr> {
    let stmt = Statement::from_string(
        DbBackend::Sqlite,
        "SELECT page_count * page_size as size FROM pragma_page_count(), pragma_page_size()"
            .to_string(),
    );

    let result = db.query_one(stmt).await?;
    if let Some(row) = result {
        let size: i64 = row.try_get("", "size")?;
        Ok(size)
    } else {
        Ok(0)
    }
}

#[command(
    slash_command,
    global_cooldown = 10,
    name_localized("zh-CN", "系统信息"),
    description_localized("zh-CN", "获取系统信息，包括系统名称、内核版本和操作系统版本"),
    ephemeral
)]
/// Fetches system information
pub async fn system_info(ctx: Context<'_>, ephemeral: Option<bool>) -> Result<(), BotError> {
    use tikv_jemalloc_ctl::{epoch, stats};
    let ephemeral = ephemeral.unwrap_or(true);
    let kernel_version = System::kernel_long_version();
    let os_version = System::long_os_version().unwrap_or_else(|| "Unknown".into());
    let e = epoch::mib()?;
    let allocated = stats::allocated::mib()?;
    e.advance()?;
    let allocated_value = allocated.read()?;
    let allocated_mb = allocated_value / 1024 / 1024; // Convert to MB
    let sys = System::new_all();
    let cpu = sys
        .cpus()
        .into_iter()
        .map(|cpu| format!("{} MHz", cpu.frequency() / 1000 / 1000))
        .collect::<Vec<_>>()
        .join(", ");
    let cpu_usage = sys.global_cpu_usage();
    let total_memory = sys.total_memory() / 1024 / 1024; // Convert to MB
    let used_memory = sys.used_memory() / 1024 / 1024; // Convert to MB
    let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
    let cached_users = ctx.cache().user_count();
    let cached_guilds = ctx.cache().guild_count();
    let cached_channels = ctx.cache().guild_channel_count();
    let rust_version = compile_time::rustc_version_str!();
    let db_size = get_db_size(DB.inner()).await? / 1024 / 1024; // Convert to MB
    let latency = ctx.ping().await;
    let metrics = tokio::runtime::Handle::current().metrics();
    let queue_count = metrics.global_queue_depth();
    let active_count = metrics.num_alive_tasks();
    let workers = metrics.num_workers();

    // Get color based on CPU usage
    let color = if cpu_usage < 50.0 {
        GREEN // Green
    } else if cpu_usage < 80.0 {
        YELLOW // Yellow
    } else {
        RED // Red
    };

    let embed = CreateEmbed::new()
        .title("🖥️ 系统信息")
        .color(color)
        // row 0
        .field("📟 OS 版本", &os_version, true)
        .field("🔧 内核版本", &kernel_version, true)
        .field("🦀 Rust 版本", rust_version, true)
        // row 1
        .field("🔳 CPU 频率", cpu, true)
        .field("🔥 CPU 使用率", format!("{:.1}%", cpu_usage), true)
        .field(
            "🧠 系统内存",
            format!(
                "{:.1}% ({} MB / {} MB)",
                memory_usage, used_memory, total_memory
            ),
            true,
        )
        // row 2
        .field("🧠 Bot 内存", format!("{} MB", allocated_mb), true)
        .field("⛁ 数据库大小", format!("{} MB", db_size), true)
        .field("⏱️ 响应延迟", format!("{} ms", latency.as_millis()), true)
        // row 3
        .field("🚦 Tokio 队列任务", queue_count.to_string(), true)
        .field("🚀 Tokio 活跃任务", active_count.to_string(), true)
        .field("🛠️ Tokio 工作线程", workers.to_string(), true)
        // row 4
        .field("👥 缓存用户数", cached_users.to_string(), true)
        .field("🌐 缓存服务器数", cached_guilds.to_string(), true)
        .field("📺 缓存频道数", cached_channels.to_string(), true)
        .timestamp(chrono::Utc::now())
        .footer(CreateEmbedFooter::new("系统监控"))
        .author(CreateEmbedAuthor::from(User::from(
            ctx.cache().current_user().clone(),
        )));

    ctx.send(CreateReply::default().embed(embed).ephemeral(ephemeral))
        .await?;

    Ok(())
}

#[command(slash_command, owners_only, ephemeral)]
pub async fn guilds_info(ctx: Context<'_>) -> Result<(), BotError> {
    let guild_ids = ctx.cache().guilds();
    // print guilds info, and bot permissions in each guild
    let message = guild_ids
        .into_iter()
        .map(async |guild_id| {
            let guild = ctx.cache().guild(guild_id).map(|g| g.to_owned())?;
            let user_id = ctx.cache().current_user().id;
            let member = guild.member(ctx, user_id).await.ok()?;
            let permissions =
                guild.user_permissions_in(guild.default_channel(member.user.id)?, &member);

            Some(format!(
                "Guild: {}\nPermissions: {}\n\n",
                guild.name,
                permissions.get_permission_names().join(", ")
            ))
        })
        .collect::<FuturesOrdered<_>>()
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n");

    if message.is_empty() {
        ctx.say("没有找到任何服务器信息。").await?;
        return Ok(());
    }
    ctx.send(
        CreateReply::default().embed(
            CreateEmbed::new()
                .title("Guilds Information")
                .description(message)
                .color(0x00FF00),
        ),
    )
    .await?;
    Ok(())
}
