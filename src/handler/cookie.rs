use const_format::formatcp;
use serenity::{async_trait, model::channel::Message, prelude::*};
use tracing::warn;

use crate::error::BotError;

pub struct CookieHandler;

#[async_trait]
impl EventHandler for CookieHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        const DECLARATION: &str = "我明白公屏发送 Cookie 的风险, Cookie 可能会被滥用，包括用于非 AIRP 用途、使用高消耗模型如 Claude Opus 等。";
        const COOKIE_PATTERN: &str = "sk-ant-sid01";
        const WARNING: &str = formatcp!(
            "❌ 我们不建议在公屏发送 Cookie, 这可能会导致 Cookie 被滥用。请谨慎处理您的 Cookie 信息。\n\
建议使用 `/submit_cookie`(English)或`/提交曲奇`(中文)命令提交 Cookie 给公益站, 以确保安全和隐私。\n\
如果您确实需要在公屏发送 Cookie, 请确保您已经了解相关风险, 在你的消息中包含以下声明:\n```{}```",
            DECLARATION
        );

        if msg.content.contains(DECLARATION) {
            return;
        }
        if msg
            .content
            .chars()
            .filter(|c| c.is_alphanumeric() || c == &'-')
            .collect::<String>()
            .contains(COOKIE_PATTERN)
        {
            let f = async move || -> Result<(), BotError> {
                let reply = msg.reply_ping(&ctx.http, WARNING).await?;
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                Ok(reply.delete(&ctx.http).await?)
            };
            if let Err(e) = f().await {
                warn!("Error handling cookie message: {}", e);
            }
        }
    }
}
