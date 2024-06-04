use axum::async_trait;
use osint_graph_shared::node::Node;

use sqlx::SqlitePool;
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Node {
    fn table() -> &'static str {
        "node"
    }

    async fn create_table(pool: &SqlitePool) -> Result<(), DBError> {
        sqlx::query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                value TEXT NOT NULL,
                updated TEXT NOT NULL,
                notes TEXT,
                pos_x INTEGER,
                pos_y INTEGER,
                FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE ON UPDATE CASCADE
            )
            ",
            Self::table()
        ))
        .execute(pool)
        .await?;
        debug!("Created table {}", Self::table());
        Ok(())
    }

    async fn save(&self, _pool: &SqlitePool) -> Result<(), DBError> {
        let querystring = format!(
            "INSERT INTO {} (id, project_id, value, updated, notes) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO
                UPDATE SET project_id = ?, value = ?, updated = ?, notes = ?;",
            Self::table()
        );
        sqlx::query(&querystring)
            .bind(self.id)
            .bind(self.project_id)
            .bind(self.value.clone())
            .bind(self.updated)
            .bind(self.notes.clone())
            .bind(self.project_id)
            .bind(self.value.clone())
            .bind(self.updated)
            .bind(self.notes.clone())
            .execute(_pool)
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

#[allow(dead_code)] // TODO: remove this
#[async_trait]
trait NodeExt {
    async fn get_by_project_id(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl NodeExt for Node {
    async fn get_by_project_id(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE project_id = ?", Self::table());
        sqlx::query_as(&querystring)
            .bind(project_id)
            .fetch_all(pool)
            .await
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use osint_graph_shared::node::NodeUpdateList;
    use osint_graph_shared::project::Project;

    use crate::storage::test_db;

    use super::*;

    #[tokio::test]
    async fn test_create_table() {
        let pool = test_db(None).await.unwrap();
        Node::create_table(&pool)
            .await
            .expect("Failed to create in-memory table for Node");
    }

    #[tokio::test]
    async fn test_crud() {
        let conn = test_db(None).await.unwrap();
        Node::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Node");

        let project_id = Uuid::new_v4();
        let id = Uuid::new_v4();
        let user = Uuid::new_v4();
        let value = "Foo".to_string();

        let project = Project {
            id: project_id,
            name: "foobar".to_string(),
            user,
            creationdate: DateTime::from(std::time::SystemTime::now()),
            last_updated: None,
            nodes: NodeUpdateList::new(),
        };

        project.save(&conn).await.expect("Failed to save project");

        let mut node = Node {
            project_id,
            id,
            value,
            updated: DateTime::from(std::time::SystemTime::now()),
            ..Default::default()
        };

        node.save(&conn).await.expect("Failed to save");

        node.notes = Some("notes".to_string());

        node.save(&conn).await.expect("Failed to update");

        node.pos_x = Some(30);
        node.pos_y = Some(54);

        node.save(&conn)
            .await
            .expect("Failed to store nod position");

        let get_ok = Node::get(&conn, &id)
            .await
            .expect("Failed to query")
            .expect("Failed to find row");
        assert_eq!(get_ok.id, id);

        let nodes = Node::get_by_project_id(&conn, project_id)
            .await
            .expect("Failed to search by project_id");
        assert!(!nodes.is_empty());

        Node::delete_by_id(&conn, id)
            .await
            .expect("Failed to delete");

        assert!(Node::get(&conn, &id)
            .await
            .expect("Failed to get")
            .is_none());
    }
}
