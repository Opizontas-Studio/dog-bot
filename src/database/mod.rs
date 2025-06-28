pub mod entities;

use clap::Parser;
use sea_orm::{Database, DatabaseConnection, DbErr};
use std::{path::Path, sync::LazyLock};

use crate::Args;

pub static DB: LazyLock<BotDatabase> = LazyLock::new(|| {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            BotDatabase::new(Args::parse().db)
                .await
                .expect("Failed to initialize database")
        })
    })
});

pub struct BotDatabase {
    db: DatabaseConnection,
}

impl BotDatabase {
    /// Get the global database instance
    pub fn get() -> &'static BotDatabase {
        &DB
    }
    pub async fn new(path: impl AsRef<Path>) -> Result<Self, DbErr> {
        let database_url = format!("sqlite://{}", path.as_ref().display());
        let db = Database::connect(&database_url).await?;

        // Initialize tables
        Self::init_tables(&db).await?;

        Ok(BotDatabase { db })
    }

    pub fn db(&self) -> &DatabaseConnection {
        &self.db
    }

    async fn init_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
        use sea_orm::{Schema, DbBackend, ConnectionTrait};
        
        let schema = Schema::new(DbBackend::Sqlite);
        
        // Create messages table using SeaORM schema builder
        let stmt = schema.create_table_from_entity(entities::Messages);
        db.execute(db.get_database_backend().build(&stmt)).await?;
        
        // Create pending_flushes table using SeaORM schema builder  
        let stmt = schema.create_table_from_entity(entities::PendingFlushes);
        db.execute(db.get_database_backend().build(&stmt)).await?;
        
        // Create indexes using SeaORM schema builder
        use sea_orm::sea_query::Index;
        
        // Messages table indexes
        let idx = Index::create()
            .if_not_exists()
            .name("idx_messages_user_guild")
            .table(entities::messages::Entity)
            .col(entities::messages::Column::UserId)
            .col(entities::messages::Column::GuildId)
            .to_owned();
        db.execute(db.get_database_backend().build(&idx)).await?;
        
        let idx = Index::create()
            .if_not_exists()
            .name("idx_messages_guild_channel")
            .table(entities::messages::Entity)
            .col(entities::messages::Column::GuildId)
            .col(entities::messages::Column::ChannelId)
            .to_owned();
        db.execute(db.get_database_backend().build(&idx)).await?;
        
        let idx = Index::create()
            .if_not_exists()
            .name("idx_messages_timestamp")
            .table(entities::messages::Entity)
            .col(entities::messages::Column::Timestamp)
            .to_owned();
        db.execute(db.get_database_backend().build(&idx)).await?;
        
        // Pending flushes table indexes
        let idx = Index::create()
            .if_not_exists()
            .name("idx_pending_flushes_notification")
            .table(entities::pending_flushes::Entity)
            .col(entities::pending_flushes::Column::NotificationId)
            .to_owned();
        db.execute(db.get_database_backend().build(&idx)).await?;
        
        let idx = Index::create()
            .if_not_exists()
            .name("idx_pending_flushes_created_at")
            .table(entities::pending_flushes::Entity)
            .col(entities::pending_flushes::Column::CreatedAt)
            .to_owned();
        db.execute(db.get_database_backend().build(&idx)).await?;

        Ok(())
    }
}
