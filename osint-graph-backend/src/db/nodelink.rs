use axum::async_trait;
use futures::TryStreamExt;
use osint_graph_shared::nodelink::NodeLink;

use sqlx::{Row, SqlitePool};
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for NodeLink {
    fn table() -> &'static str {
        "nodelink"
    }

    async fn create_table(pool: &SqlitePool) -> Result<(), DBError> {
        sqlx::query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                left TEXT NOT NULL,
                right TEXT NOT NULL,
                project_id TEXT NOT NULL,
                linktype TEXT NOT NULL,
                FOREIGN KEY (project_id) REFERENCES project(id) ON DELETE CASCADE ON UPDATE CASCADE,
                FOREIGN KEY (left) REFERENCES node(id) ON DELETE CASCADE ON UPDATE CASCADE,
                FOREIGN KEY (right) REFERENCES node(id) ON DELETE CASCADE ON UPDATE CASCADE
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
            "INSERT INTO {} (id, project_id, left, right, linktype) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO
                UPDATE SET project_id = ?, left = ?, right = ?, linktype = ?;",
            Self::table()
        );
        sqlx::query(&querystring)
            .bind(self.id)
            .bind(self.project_id)
            .bind(self.left)
            .bind(self.right)
            .bind(self.linktype)
            .bind(self.project_id)
            .bind(self.left)
            .bind(self.right)
            .bind(self.linktype)
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
impl NodeExt for NodeLink {
    async fn get_by_project_id(pool: &SqlitePool, project_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE project_id = ?", Self::table());
        let mut rows = sqlx::query(&querystring).bind(project_id).fetch(pool);

        let mut nodes = Vec::new();

        while let Some(row) = rows.try_next().await? {
            // let linktype: u8 = row.get("linktype");
            nodes.push(Self {
                project_id: row.get("project_id"),
                id: row.get("id"),
                left: row.get("left"),
                right: row.get("right"),
                linktype: row.get("linktype"),
            });
        }

        Ok(nodes)
    }
}

#[allow(dead_code)] // TODO: remove this
#[async_trait]
trait NodeLinkExt {
    async fn get_by_node_id(pool: &SqlitePool, node_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl NodeLinkExt for NodeLink {
    async fn get_by_node_id(pool: &SqlitePool, node_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!(
            "SELECT id, project_id, left, right, linktype FROM {} WHERE left = ? OR right = ?",
            Self::table()
        );
        let mut rows = sqlx::query(&querystring)
            .bind(node_id)
            .bind(node_id)
            .fetch(pool);

        let mut nodes = Vec::new();

        while let Some(row) = rows.try_next().await? {
            nodes.push(Self {
                project_id: row.get("project_id"),
                id: row.get("id"),
                left: row.get("left"),
                right: row.get("right"),
                linktype: row.get("linktype"),
            });
        }

        Ok(nodes)
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use osint_graph_shared::node::{Node, NodeUpdateList};
    use osint_graph_shared::nodelink::LinkType;
    use osint_graph_shared::project::Project;

    use crate::storage::start_db;

    use super::*;

    #[tokio::test]
    async fn test_create_table() {
        let conn = start_db(None,None).await.unwrap();
        NodeLink::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Node");
    }

    #[tokio::test]
    async fn test_crud() {
        let conn = start_db(None,None).await.unwrap();
        NodeLink::create_table(&conn)
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

        let node = Node {
            project_id,
            id,
            value,
            updated: DateTime::from(std::time::SystemTime::now()),
            ..Default::default()
        };

        node.save(&conn).await.expect("Failed to save node");

        let mut link = NodeLink::new(node.id, node.id, project_id, LinkType::Omni);

        link.save(&conn).await.expect("Failed to save link");

        link.linktype = LinkType::Directional;

        link.save(&conn).await.expect("Failed to update link");

        let links = NodeLink::get_by_project_id(&conn, project_id)
            .await
            .expect("Failed to get_by_project_id");
        assert!(!links.is_empty());

        let links = NodeLink::get_by_node_id(&conn, node.id)
            .await
            .expect("Failed to get_by_node_id");
        assert!(!links.is_empty());

        NodeLink::delete_by_id(&conn, link.id)
            .await
            .expect("Failed to delete");

        assert!(NodeLink::get(&conn, &link.id)
            .await
            .expect("Failed to get")
            .is_none());

        let links = NodeLink::get_by_project_id(&conn, project_id)
            .await
            .expect("Failed to get_by_project_id");
        assert!(links.is_empty());

        let links = NodeLink::get_by_node_id(&conn, node.id)
            .await
            .expect("Failed to get_by_node_id");
        assert!(links.is_empty());

        let _: NodeLink = serde_json::from_value(serde_json::json!({
            "id" : Uuid::new_v4(),
            "left" : Uuid::new_v4(),
            "right" : Uuid::new_v4(),
            "project_id" : Uuid::new_v4(),
            "linktype" : "Directional"
        }))
        .expect("Failed to deserialise NodeLink");
    }
}
