use crate::error::BotError;
use tracing::info;
type Context<'a> = poise::Context<'a, (), BotError>;

struct Data {}

#[poise::command(prefix_command)]
async fn register_volunteer(ctx: Context<'_>) -> Result<(), BotError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

fn option() -> poise::FrameworkOptions<(), BotError> {
    poise::FrameworkOptions {
        commands: vec![register_volunteer()],
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
        skip_checks_for_owners: true,
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
