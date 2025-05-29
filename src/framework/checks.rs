/**
Poise supports several pre-command checks (sorted by order of execution):
- owners_only
- required_permissions
- required_bot_permissions
- global check function
- command-specific check function
- cooldowns
*/
use crate::error::BotError;
use tracing::info;
type Context<'a> = poise::Context<'a, (), BotError>;

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.framework().shard_manager().shutdown_all().await;
    Ok(())
}

/// A moderator-only command, using required_permissions
#[poise::command(
    slash_command,
    // Multiple permissions can be OR-ed together with `|` to make them all required
    required_permissions = "MANAGE_MESSAGES | MANAGE_THREADS",
)]
pub async fn modonly(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("You are a mod because you were able to invoke this command")
        .await?;
    Ok(())
}

/// Crab party... only for "Ferris"!
#[poise::command(slash_command)]
pub async fn ferrisparty(ctx: Context<'_>) -> Result<(), BotError> {
    let response = "```\n".to_owned()
        + &r"    _~^~^~_
\) /  o o  \ (/
  '_   ¬   _'
  | '-----' |
"
        .repeat(3)
        + "```";
    ctx.say(response).await?;
    Ok(())
}

/// Add two numbers
#[poise::command(
    track_edits,
    slash_command,
    // All cooldowns in seconds
    global_cooldown = 1,
    user_cooldown = 5,
    guild_cooldown = 2,
    channel_cooldown = 2,
    member_cooldown = 3,
)]
pub async fn cooldowns(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("You successfully called the command").await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn minmax(
    ctx: Context<'_>,
    #[min = 15]
    #[max = 28.765]
    value: f32,
) -> Result<(), BotError> {
    ctx.say(format!("You submitted number {}", value)).await?;
    Ok(())
}

/// Get the guild name (guild-only)
#[poise::command(slash_command, guild_only)]
pub async fn get_guild_name(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say(format!(
        "The name of this guild is: {}",
        ctx.partial_guild().await.unwrap().name
    ))
    .await?;

    Ok(())
}

/// A dm-only command
#[poise::command(slash_command, dm_only)]
pub async fn only_in_dms(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("This is a dm channel").await?;

    Ok(())
}

/// Only runs on NSFW channels
#[poise::command(slash_command, nsfw_only)]
pub async fn lennyface(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("( ͡° ͜ʖ ͡°)").await?;

    Ok(())
}

/// Utilizes the permissions v2 `default_member_permissions` field
#[poise::command(slash_command, default_member_permissions = "ADMINISTRATOR")]
pub async fn permissions_v2(ctx: Context<'_>) -> Result<(), BotError> {
    ctx.say("Whoop! You're authorized!").await?;

    Ok(())
}

#[poise::command(prefix_command, owners_only, hide_in_help)]
pub async fn register(ctx: Context<'_>) -> Result<(), BotError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

fn option() -> poise::FrameworkOptions<(), BotError> {
    poise::FrameworkOptions {
        commands: vec![
            shutdown(),
            modonly(),
            ferrisparty(),
            cooldowns(),
            minmax(),
            get_guild_name(),
            only_in_dms(),
            lennyface(),
            permissions_v2(),
            register(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: None,
            ..Default::default()
        },
        on_error: |error| {
            Box::pin(async {
                poise::builtins::on_error(error).await.unwrap_or_else(|e| {
                    tracing::error!("Error while handling error: {}", e);
                })
            })
        },
        pre_command: |ctx| {
            Box::pin(async move { info!("Executing command {}", ctx.command().name) })
        },
        post_command: |ctx| {
            Box::pin(async move { info!("Finished executing command {}", ctx.command().name) })
        },
        ..Default::default()
    }
}

pub fn framework() -> poise::Framework<(), BotError> {
    poise::Framework::builder()
        .setup(|_, _, _| {
            Box::pin(async move {
                // This is run when the framework is set up
                info!("Framework has been set up!");
                Ok(())
            })
        })
        .options(option())
        .build()
}
