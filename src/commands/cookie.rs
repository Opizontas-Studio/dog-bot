use poise::{CreateReply, command};
use serenity::all::MessageBuilder;
use snafu::{ResultExt, whatever};

use super::Context;
use crate::error::BotError;

#[command(
    slash_command,
    name_localized("zh-CN", "提交曲奇"),
    description_localized("zh-CN", "提交曲奇给求封站"),
    ephemeral
)]
/// Submits a cookie to account banning site
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
    let Some(url) = ctx.data().cfg.load().cookie_endpoint.to_owned() else {
        ctx.say("Cookie endpoint is not configured.").await?;
        whatever!("Cookie endpoint is not configured");
    };
    let reply = ctx.say("Submitting cookie...").await?;
    let client = reqwest::Client::new();
    if let Err(e) = client
        .post(
            url.join("api/cookie")
                .whatever_context::<&str, BotError>("Failed to construct cookie submission URL")?,
        )
        .json(&CookieSubmission { cookie })
        .bearer_auth(ctx.data().cfg.load().cookie_secret.to_owned())
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
                        .push_bold_safe(e.status().map_or_else(|| e.to_string(), |s| s.to_string()))
                        .build(),
                ),
            )
            .await?;
        Err(e.into())
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
