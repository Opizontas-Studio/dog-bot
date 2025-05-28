use std::collections::HashSet;

use crate::error::BotError;
use poise::command;
use serenity::{all::UserId, futures::lock::Mutex};
use tracing::info;
type Context<'a> = poise::Context<'a, Data, BotError>;

#[derive(Debug, Default)]
pub struct BoundedSet<T> {
    items: HashSet<T>,
    max_size: usize,
}

impl<T> BoundedSet<T>
where
    T: Eq + std::hash::Hash + Clone,
{
    pub fn new(max_size: usize) -> Self {
        Self {
            items: HashSet::new(),
            max_size,
        }
    }

    pub fn push(&mut self, item: T) -> Result<(), T> {
        if self.items.len() >= self.max_size {
            Err(item) // Return the item back if at capacity
        } else {
            self.items.insert(item);
            Ok(())
        }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.max_size
    }
}

#[derive(Debug, Default)]
pub struct Data {
    supervisors: Mutex<BoundedSet<UserId>>,
}

#[command(prefix_command, owners_only, hide_in_help)]
async fn register_supervisor_framework(ctx: Context<'_>) -> Result<(), BotError> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

fn option() -> poise::FrameworkOptions<Data, BotError> {
    poise::FrameworkOptions {
        commands: vec![register_supervisor_framework()],
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

pub fn framework() -> poise::Framework<Data, BotError> {
    poise::Framework::builder()
        .setup(|_, _, _| {
            Box::pin(async move {
                // This is run when the framework is set up
                info!("Supervisors framework has been set up!");
                Ok(Default::default())
            })
        })
        .options(option())
        .build()
}
