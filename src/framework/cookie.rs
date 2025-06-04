use crate::error::BotError;
use poise::command;

use super::Context;
pub mod command {
    use poise::CreateReply;
    use serenity::all::MessageBuilder;
    use snafu::{ResultExt, whatever};

    use super::*;
    #[command(
        slash_command,
        global_cooldown = 10,
        name_localized("zh-CN", "提交曲奇"),
        description_localized("zh-CN", "提交曲奇给任梓乐的公益站")
    )]
    /// Submits a cookie to rzline's charity site.
    pub async fn submit_cookie(ctx: Context<'_>, cookie: String) -> Result<(), BotError> {
        #[derive(serde::Serialize)]
        struct CookieSubmission {
            cookie: String,
        }
        ctx.defer_ephemeral().await?;
        let Some(url) = crate::config::BOT_CONFIG.cookie_endpoint.as_ref() else {
            ctx.say("Cookie endpoint is not configured.").await?;
            whatever!("Cookie endpoint is not configured");
        };
        let reply = ctx.say("Submitting cookie...").await?;
        let client = reqwest::Client::new();
        if let Err(e) = client
            .post(
                url.to_owned()
                    .join("api/cookie")
                    .whatever_context::<&str, BotError>(
                        "Failed to construct cookie submission URL",
                    )?,
            )
            .json(&CookieSubmission { cookie })
            .bearer_auth(crate::config::BOT_CONFIG.cookie_secret.to_owned())
            .send()
            .await
            .and_then(|res| res.error_for_status())
        {
            reply
                .edit(
                    ctx,
                    CreateReply::default().content(
                        MessageBuilder::new()
                            .push("❌ Failed to submit cookie: ")
                            .push_bold_safe(e.status().unwrap_or_default().as_str())
                            .build(),
                    ),
                )
                .await?;
            Err(e).whatever_context("Failed to submit cookie")
        } else {
            reply
                .edit(
                    ctx,
                    CreateReply::default().content("✅ Cookie submitted successfully!"),
                )
                .await?;
            Ok(())
        }
    }
}
