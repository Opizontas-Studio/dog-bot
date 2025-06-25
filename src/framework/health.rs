use crate::error::BotError;
use poise::command;
use snafu::whatever;
use sysinfo::System;
use tracing::error;

use super::Context;
pub mod command {
    use poise::CreateReply;
    use serenity::all::{CreateEmbed, ModelError};
    use snafu::OptionExt;

    use super::*;
    #[command(
        slash_command,
        global_cooldown = 10,
        name_localized("zh-CN", "健康状态"),
        description_localized("zh-CN", "获取机器的健康状态，包括 CPU 和内存使用情况"),
        ephemeral
    )]
    /// Fetches the health status of machine, including CPU and memory usage.
    pub async fn health(ctx: Context<'_>) -> Result<(), BotError> {
        let mut sys = System::new_all();
        sys.refresh_all();
        let cpu_usage = sys.global_cpu_usage();
        let total_memory = sys.total_memory() / 1024 / 1024; // Convert to MB
        let used_memory = sys.used_memory() / 1024 / 1024; // Convert to MB
        let memory_usage = (used_memory as f64 / total_memory as f64) * 100.0;
        let cached_users = ctx.cache().user_count();
        let settings = ctx.cache().settings().to_owned();
        let message = format!(
            "CPU Usage: {:.2}%\nMemory Usage: {:.2}%\nUsed Memory: {} MB\nTotal Memory: {} MB\nCached Users: {}\nSettings: {:?}",
            cpu_usage, memory_usage, used_memory, total_memory, cached_users, settings
        );
        ctx.say(message).await?;
        Ok(())
    }

    #[command(slash_command, subcommands("status", "journal"))]
    pub async fn systemd(_: Context<'_>) -> Result<(), BotError> {
        Ok(())
    }

    #[command(
        slash_command,
        global_cooldown = 10,
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
        global_cooldown = 10,
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

    #[command(
        slash_command,
        name_localized("zh-CN", "系统信息"),
        description_localized("zh-CN", "获取系统信息，包括系统名称、内核版本和操作系统版本"),
        ephemeral
    )]
    /// Fetches system information such as system name, kernel version, and OS version.
    pub async fn system_info(ctx: Context<'_>) -> Result<(), BotError> {
        let sys_name = System::name().unwrap_or("Unknown".into());
        let kernel_version = System::kernel_long_version();
        let os_version = System::long_os_version().unwrap_or("Unknown".into());
        let message = format!(
            "System Name: {}\nKernel Version: {}\nOS Version: {}",
            sys_name, kernel_version, os_version
        );
        ctx.say(message).await?;
        Ok(())
    }

    #[command(slash_command, owners_only, ephemeral)]
    pub async fn guilds_info(ctx: Context<'_>) -> Result<(), BotError> {
        let guild_ids = ctx.cache().guilds();
        // print guilds info, and bot permissions in each guild
        let mut message = String::new();
        for guild_id in guild_ids {
            let Some(guild) = ctx.cache().guild(guild_id).map(|g| g.to_owned()) else {
                continue;
            };
            let user_id = ctx.cache().current_user().id;
            let member = guild.member(ctx, user_id).await?;
            let permissions = guild.user_permissions_in(
                guild
                    .default_channel(member.user.id)
                    .whatever_context::<&str, BotError>("No channel")?,
                &member,
            );

            message.push_str(&format!(
                "Guild: {}\nPermissions: {}\n\n",
                guild.name,
                permissions.get_permission_names().join(", ")
            ));
        }
        if message.is_empty() {
            message = "No guilds found or no permissions available.".to_string();
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
}
