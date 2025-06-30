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
                    .col(big_unsigned_uniq(Messages::MessageId).primary_key())
                    .col(big_unsigned(Messages::UserId).not_null())
                    .col(big_unsigned(Messages::GuildId).not_null())
                    .col(big_unsigned(Messages::ChannelId).not_null())
                    .col(
                        timestamp_with_time_zone(Messages::Timestamp)
                            .default(Expr::current_timestamp()),
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
                    .col(big_unsigned_uniq(PendingFlushes::MessageId).primary_key())
                    .col(big_unsigned_uniq(PendingFlushes::NotificationId))
                    .col(big_unsigned(PendingFlushes::ChannelId))
                    .col(big_unsigned(PendingFlushes::ToiletId))
                    .col(big_unsigned(PendingFlushes::AuthorId))
                    .col(big_unsigned(PendingFlushes::FlusherId))
                    .col(big_unsigned(PendingFlushes::ThresholdCount).default(Expr::value(2)))
                    .col(
                        timestamp_with_time_zone(PendingFlushes::CreatedAt)
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
