use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create messages table
        manager
            .create_table(
                Table::create()
                    .table(Messages::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Messages::MessageId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Messages::UserId).big_integer().not_null())
                    .col(ColumnDef::new(Messages::GuildId).big_integer().not_null())
                    .col(ColumnDef::new(Messages::ChannelId).big_integer().not_null())
                    .col(
                        ColumnDef::new(Messages::Timestamp)
                            .timestamp_with_time_zone()
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for messages table
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_messages_user_guild")
                    .table(Messages::Table)
                    .col(Messages::UserId)
                    .col(Messages::GuildId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_messages_guild_channel")
                    .table(Messages::Table)
                    .col(Messages::GuildId)
                    .col(Messages::ChannelId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_messages_timestamp")
                    .table(Messages::Table)
                    .col(Messages::Timestamp)
                    .to_owned(),
            )
            .await?;

        // Create pending_flushes table
        manager
            .create_table(
                Table::create()
                    .table(PendingFlushes::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(PendingFlushes::MessageId)
                            .big_integer()
                            .not_null()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::NotificationId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::ChannelId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::ToiletId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::AuthorId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::FlusherId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::ThresholdCount)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(PendingFlushes::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // Create indexes for pending_flushes table
        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_pending_flushes_notification")
                    .table(PendingFlushes::Table)
                    .col(PendingFlushes::NotificationId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .if_not_exists()
                    .name("idx_pending_flushes_created_at")
                    .table(PendingFlushes::Table)
                    .col(PendingFlushes::CreatedAt)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Messages::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(PendingFlushes::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Messages {
    Table,
    MessageId,
    UserId,
    GuildId,
    ChannelId,
    Timestamp,
}

#[derive(DeriveIden)]
enum PendingFlushes {
    Table,
    MessageId,
    NotificationId,
    ChannelId,
    ToiletId,
    AuthorId,
    FlusherId,
    ThresholdCount,
    CreatedAt,
}
