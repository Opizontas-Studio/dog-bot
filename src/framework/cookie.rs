use crate::{config::BOT_CONFIG, error::BotError};
use poise::{CreateReply, command};
use serenity::all::MessageBuilder;
use snafu::{ResultExt, whatever};

use super::Context;
pub mod command {
    use super::*;
    #[command(
        slash_command,
        name_localized("zh-CN", "提交曲奇"),
        description_localized("zh-CN", "提交曲奇给 sukaka 的公益站"),
        ephemeral
    )]
    /// Submits a cookie to sukaka's charity site.
    pub async fn submit_cookie(
        ctx: Context<'_>,
        #[name_localized("zh-CN", "曲奇")]
        #[description_localized("zh-CN", "要提交的曲奇内容, 格式要求很宽松")]
        #[description = "The cookie content to submit, format is quite flexible"]
        cookie: String,
    ) -> Result<(), BotError> {
        #[derive(serde::Serialize)]
        struct CookieSubmission {
            cookie: String,
        }
        let cfg = BOT_CONFIG.load();
        let Some(url) = cfg.cookie_endpoint.as_ref() else {
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
            .bearer_auth(cfg.cookie_secret.to_owned())
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
                            .push_bold_safe(
                                e.status().map_or("Unknown error".to_string(), |s| {
                                    s.as_str().to_string()
                                }),
                            )
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
