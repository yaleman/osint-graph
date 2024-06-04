use std::io::ErrorKind;
use std::str::FromStr;
use std::time::Duration;

use axum::async_trait;
use osint_graph_shared::node::Node;
use osint_graph_shared::nodelink::NodeLink;
use osint_graph_shared::project::Project;
use sqlx::sqlite::SqliteConnectOptions;
use sqlx::ConnectOptions;

use tracing::debug;
use uuid::Uuid;

use sqlx::SqlitePool;

pub async fn new() -> Result<SqlitePool, std::io::Error> {
    let db_path = match std::env::var("OSINT_GRAPH_DB_PATH") {
        // If the OSINT_GRAPH_DB_PATH environment variable is set, use that.
        Ok(path) => path,

        // Otherwise, use the default path.
        Err(_) => shellexpand::tilde("~/.cache/osint-graph.sqlite3").to_string(),
    };

    let db_url = format!("sqlite://{db_path}?mode=rwc");
    debug!("Opening Database: {db_url}");

    let options = match SqliteConnectOptions::from_str(&db_url) {
        Ok(value) => value,
        Err(error) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("connection failed: {error:?}"),
            ))
        }
    };
    let options =
            // let options = if config_reader.sql_log_statements {
                // options.log_statements(log::LevelFilter::Trace)
            // } else {
                options.log_statements(log::LevelFilter::Off);
    // };

    let options = options.log_slow_statements(log::LevelFilter::Warn, Duration::from_secs(1));

    let conn = SqlitePool::connect_with(options).await.map_err(|err| {
        std::io::Error::new(ErrorKind::Other, format!("connection failed: {err:?}"))
    })?;

    create_tables(&conn).await?;

    Ok(conn)
}

pub async fn create_tables(conn: &SqlitePool) -> Result<(), std::io::Error> {
    Project::create_table(conn)
        .await
        .expect("Failed to create Project table");
    Node::create_table(conn)
        .await
        .expect("Failed to create Node table");
    NodeLink::create_table(conn)
        .await
        .expect("Failed to create NodeLink table");

    Ok(())
}

#[cfg(test)]
pub async fn test_db(url: Option<String>) -> Result<SqlitePool, std::io::Error> {
    let db_url = format!(
        "sqlite://{}?mode=rwc",
        url.unwrap_or(":memory:".to_string())
    );
    debug!("Opening Database: {db_url}");

    let options = match SqliteConnectOptions::from_str(&db_url) {
        Ok(value) => value,
        Err(error) => {
            return Err(std::io::Error::new(
                ErrorKind::Other,
                format!("connection failed: {error:?}"),
            ))
        }
    }
    .log_statements(log::LevelFilter::Trace)
    .log_slow_statements(log::LevelFilter::Warn, Duration::from_micros(100));

    let conn = SqlitePool::connect_with(options).await.map_err(|err| {
        std::io::Error::new(ErrorKind::Other, format!("connection failed: {err:?}"))
    })?;

    create_tables(&conn).await?;

    Ok(conn)
}

//     pub fn set(&mut self, _key: &str, _value: &str) -> Result<(), std::io::Error> {
//         todo!();
//         // let write_txn = self.db.begin_write()?;
//         // {
//         //     let mut table = write_txn.open_table(TABLE)?;
//         //     table.insert(key, value)?;
//         // }
//         // write_txn.commit()?;
//         // Ok(())
//     }

//     pub fn get(&self, _key: &str) -> Result<Option<String>, std::io::Error> {
//         todo!();
//         // let read_txn = self.db.begin_read()?;
//         // let table = read_txn.open_table(TABLE)?;

//         // let res = table.get(key)?.map(|v| v.value().to_string());
//         // Ok(res)
//     }

//     pub fn list_projects(&self) -> Result<Vec<Project>, std::io::Error> {
//         todo!();

//         // let read_txn = self.db.begin_read()?;
//         // let table = match read_txn.open_table(PROJECT_TABLE) {
//         //     Ok(val) => val,
//         //     Err(err) => match err {
//         //         TableError::TypeDefinitionChanged { .. }
//         //         | TableError::TableAlreadyOpen(_, _)
//         //         | TableError::Storage(_)
//         //         | TableError::TableIsNotMultimap(_)
//         //         | TableError::TableIsMultimap(_)
//         //         | TableError::TableTypeMismatch { .. } => return Err(err.into()),
//         //         // if the table doesn't exist we haven't saved to it yet, so there's no projects.
//         //         TableError::TableDoesNotExist(_) => return Ok(Vec::new()),

//         //         _ => {
//         //             error!("Failed to connect to table: {:?}", err);
//         //             return Ok(Vec::new());
//         //         }
//         //     },
//         // };

//         // let res = table
//         //     .iter()?
//         //     .map(|row| {
//         //         let (_uuid, row) = row.unwrap();
//         //         let row_value = row.value();
//         //         // eprintln!("Got uuid={} data={}", uuid.value(), row.value());
//         //         serde_json::from_str(row_value).expect("Failed to deserialize value")
//         //     })
//         //     .collect();
//         // Ok(res)
//     }
// }

// #[test]
// fn test_db_writethrough() {
//     let mut storage = test_db(None);

//     storage.set("test", "test").unwrap();

//     assert_eq!(storage.get("test").unwrap(), Some("test".to_string()));
//     assert!(storage.get("foo").unwrap().is_none());
// }

#[derive(Debug)]
pub enum DBError {
    SqlxError(sqlx::Error),
    IoError(std::io::Error),
    Serde(serde_json::Error),
}

impl From<sqlx::Error> for DBError {
    fn from(err: sqlx::Error) -> Self {
        DBError::SqlxError(err)
    }
}

impl From<serde_json::Error> for DBError {
    fn from(value: serde_json::Error) -> Self {
        DBError::Serde(value)
    }
}

#[async_trait]
pub trait DBEntity {
    fn table() -> &'static str;

    async fn create_table(pool: &SqlitePool) -> Result<(), DBError>;

    async fn save(&self, pool: &SqlitePool) -> Result<(), DBError>
    where
        Self: Sized;

    async fn delete_by_id(pool: &SqlitePool, id: Uuid) -> Result<(), crate::storage::DBError> {
        let querystring = format!("DELETE from {} where id = ?", Self::table());

        sqlx::query(&querystring).bind(id).execute(pool).await?;
        Ok(())
    }

    async fn get(pool: &SqlitePool, id: &Uuid) -> Result<Option<Self>, DBError>
    where
        Self: Sized;
}
