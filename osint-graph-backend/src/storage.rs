use std::path::PathBuf;

use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::debug;

use crate::migration::Migrator;

// Start the database
pub async fn new(db_path: &PathBuf) -> Result<DatabaseConnection, std::io::Error> {
    start_db(Some(db_path)).await
}

pub async fn start_db(db_path: Option<&PathBuf>) -> Result<DatabaseConnection, std::io::Error> {
    let db_url = match db_path {
        Some(path) => {
            let path = path.to_string_lossy().to_string();
            let path = shellexpand::tilde(&path);

            debug!(
                path = path.to_string(),
                "Database path after tilde expansion"
            );
            format!("sqlite://{}?mode=rwc", path)
        }
        None => "sqlite::memory:".to_string(),
    };
    debug!("Opening Database: {db_url}");

    let conn = Database::connect(&db_url)
        .await
        .map_err(|err| std::io::Error::other(format!("connection failed: {err:?}")))?;

    // Enable foreign key constraints
    use sea_orm::ConnectionTrait;
    let _ = conn
        .execute(sea_orm::Statement::from_string(
            sea_orm::DatabaseBackend::Sqlite,
            "PRAGMA foreign_keys = ON".to_string(),
        ))
        .await
        .map_err(|err| std::io::Error::other(format!("Failed to enable foreign keys: {err:?}")))?;

    // Run migrations
    Migrator::up(&conn, None)
        .await
        .map_err(|err| std::io::Error::other(format!("Migration failed: {err:?}")))?;

    Ok(conn)
}

#[derive(Debug)]
pub enum DBError {
    SeaOrmError(DbErr),
    IoError(std::io::Error),
    Serde(serde_json::Error),
    Other(String),
}

impl From<DbErr> for DBError {
    fn from(err: DbErr) -> Self {
        DBError::SeaOrmError(err)
    }
}

impl From<serde_json::Error> for DBError {
    fn from(value: serde_json::Error) -> Self {
        DBError::Serde(value)
    }
}

impl From<std::io::Error> for DBError {
    fn from(value: std::io::Error) -> Self {
        DBError::IoError(value)
    }
}
