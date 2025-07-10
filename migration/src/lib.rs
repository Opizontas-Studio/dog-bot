pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20250704_012322_add_flush_reason;
mod m20250710_000001_optimize_channel_stats;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20250704_012322_add_flush_reason::Migration),
            Box::new(m20250710_000001_optimize_channel_stats::Migration),
        ]
    }
}
