use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add composite index for efficient channel stats queries
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_messages_guild_timestamp_channel")
                    .table(Messages::Table)
                    .col(Messages::GuildId)
                    .col(Messages::Timestamp)
                    .col(Messages::ChannelId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_messages_guild_timestamp_channel")
                    .table(Messages::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    GuildId,
    ChannelId,
    Timestamp,
}
