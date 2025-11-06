pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tables;
mod m20250105_000001_insert_default_inbox_project;
mod m20251106_000001_drop_attachments_column_nodes;
mod m20251106_000002_create_sessions;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20250105_000001_insert_default_inbox_project::Migration),
            Box::new(m20251106_000001_drop_attachments_column_nodes::Migration),
            Box::new(m20251106_000002_create_sessions::Migration),
        ]
    }
}
