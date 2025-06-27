use axum::async_trait;
use osint_graph_shared::project::Project;
use sqlx::SqlitePool;
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Project {
    fn table() -> &'static str {
        "project"
    }

    async fn create_table(pool: &SqlitePool) -> Result<(), DBError> {
        sqlx::query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                user TEXT NOT NULL,
                creationdate TEXT NOT NULL,
                last_updated TEXT,
                nodes TEXT
            )
            ",
            Self::table()
        ))
        .execute(pool)
        .await?;
        debug!("Created table {}", Self::table());
        Ok(())
    }

    async fn save(&self, pool: &SqlitePool) -> Result<(), DBError> {
        let querystring = format!(
            "INSERT INTO {} (id, name, user, creationdate, last_updated, nodes ) VALUES (?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO
            UPDATE SET name = ?, user = ?, creationdate = ?, last_updated = ? ",
            Self::table()
        );
        let nodes_encoded = serde_json::to_string(&self.nodes)?;

        sqlx::query(&querystring)
            .bind(self.id)
            .bind(self.name.clone())
            .bind(self.user)
            .bind(self.creationdate)
            .bind(self.last_updated)
            .bind(&nodes_encoded)
            .bind(self.name.clone())
            .bind(self.user)
            .bind(self.creationdate)
            .bind(self.last_updated)
            .bind(&nodes_encoded)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn get(pool: &SqlitePool, id: &Uuid) -> Result<Option<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE id = ?", Self::table());

        sqlx::query_as(&querystring)
            .bind(id)
            .fetch_optional(pool)
            .await
            .map_err(|e| e.into())
    }
}

#[async_trait]
pub trait DBProjectExt {
    async fn get_all(conn: &SqlitePool) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl DBProjectExt for Project {
    async fn get_all(conn: &SqlitePool) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {}", Project::table());
        sqlx::query_as(&querystring)
            .fetch_all(conn)
            .await
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;

    use crate::storage::start_db;

    use super::*;

    #[tokio::test]
    async fn test_node_create_table() {
        let conn = start_db(None, None).await.unwrap();
        Project::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Project");
    }

    #[tokio::test]
    async fn test_crud() {
        let conn = start_db(None, None).await.unwrap();
        Project::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Project");

        let id = Uuid::new_v4();
        let user = Uuid::new_v4();
        let name = "Foo".to_string();
        let mut project = Project::default().name(name).user(user).id(id);
        project.save(&conn).await.expect("Failed to save");

        project.updated();

        project.save(&conn).await.expect("Failed to update");

        project
            .nodes
            .insert(Uuid::new_v4(), DateTime::from(std::time::SystemTime::now()));

        project
            .nodes
            .insert(Uuid::new_v4(), DateTime::from(std::time::SystemTime::now()));

        project.save(&conn).await.expect("Failed to save nodes");

        let get_ok = Project::get(&conn, &id)
            .await
            .expect("Failed to query")
            .expect("Failed to find row");
        assert_eq!(get_ok.id, id);

        Project::delete_by_id(&conn, id)
            .await
            .expect("Failed to delete");

        assert!(Project::get(&conn, &id)
            .await
            .expect("Failed to get")
            .is_none());
    }
}
