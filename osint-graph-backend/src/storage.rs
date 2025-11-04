use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use tracing::debug;
use uuid::Uuid;

use crate::migration::Migrator;

// Start the database
pub async fn new() -> Result<DatabaseConnection, std::io::Error> {
    let db_path = match std::env::var("OSINT_GRAPH_DB_PATH") {
        // If the OSINT_GRAPH_DB_PATH environment variable is set, use that.
        Ok(path) => path,
        // Otherwise, use the default path.
        Err(_) => shellexpand::tilde("~/.cache/osint-graph.sqlite3").to_string(),
    };

    start_db(Some(db_path)).await
}

pub async fn start_db(db_path: Option<String>) -> Result<DatabaseConnection, std::io::Error> {
    let db_url = match db_path {
        Some(path) => format!("sqlite://{}?mode=rwc", path),
        None => "sqlite::memory:".to_string(),
    };
    // let db_path = db_path.unwrap_or(":memory:".to_string());
    // let db_url = format!("sqlite://{}?mode=rwc", db_path);
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

    // // Create default project if it doesn't exist
    // create_default_project(&conn).await?;

    Ok(conn)
}

// async fn create_default_project(conn: &DatabaseConnection) -> Result<(), std::io::Error> {
//     let default_project_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
//         .map_err(|e| std::io::Error::other(format!("Invalid UUID: {}", e)))?;

//     // Check if default project already exists
//     if let Ok(Some(_)) = project::Entity::find()
//         .filter(project::Column::Id.eq(default_project_id))
//         .one(conn)
//         .await
//     {
//         debug!("Default project already exists");
//         return Ok(());
//     }

//     let default_project = Project {
//         id: default_project_id,
//         name: "Default Project".to_string(),
//         user: Uuid::new_v4(), // Generate a random user ID for demo
//         creationdate: std::time::SystemTime::now().into(),
//         last_updated: None,
//         nodes: NodeUpdateList::new(),
//         description: None,
//         tags: Vec::new(),
//     };

//     let default_project = project::ActiveModel::from(default_project);

//     default_project
//         .save(conn)
//         .await
//         .map_err(|e| std::io::Error::other(format!("Failed to create default project: {:?}", e)))?;

//     debug!("Created default project with ID: {}", default_project_id);
//     Ok(())
// }

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

// Legacy DBEntity trait for backward compatibility during migration
// This will be removed once all code is migrated to use SeaORM entities directly
#[axum::async_trait]
pub trait DBEntity {
    fn table() -> &'static str;

    async fn create_table(conn: &DatabaseConnection) -> Result<(), DBError>;

    async fn save(&self, conn: &DatabaseConnection) -> Result<(), DBError>
    where
        Self: Sized;

    async fn delete_by_id(
        conn: &DatabaseConnection,
        id: Uuid,
    ) -> Result<(), crate::storage::DBError> {
        use sea_orm::ConnectionTrait;
        let querystring = format!("DELETE from {} where id = ?", Self::table());

        let _ = conn
            .execute(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
                vec![id.to_string().into()],
            ))
            .await?;
        Ok(())
    }

    async fn get(conn: &DatabaseConnection, id: &Uuid) -> Result<Option<Self>, DBError>
    where
        Self: Sized;
}
