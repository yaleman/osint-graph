use axum::async_trait;
use osint_graph_shared::nodelink::NodeLink;

use sea_orm::{DatabaseConnection, FromQueryResult};
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for NodeLink {
    fn table() -> &'static str {
        "nodelink"
    }

    async fn create_table(_conn: &DatabaseConnection) -> Result<(), DBError> {
        // Tables are now created via migrations
        debug!("Skipping create table - using migrations");
        Ok(())
    }

    async fn save(&self, conn: &DatabaseConnection) -> Result<(), DBError> {
        let querystring = format!(
            "INSERT INTO {} (id, project_id, left, right, linktype) VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(id) DO
                UPDATE SET project_id = ?, left = ?, right = ?, linktype = ?;",
            Self::table()
        );

        sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![
                self.id.to_string().into(),
                self.project_id.to_string().into(),
                self.left.to_string().into(),
                self.right.to_string().into(),
                self.linktype.clone().into(),
                self.project_id.to_string().into(),
                self.left.to_string().into(),
                self.right.to_string().into(),
                self.linktype.clone().into(),
            ],
        )
        .execute(conn)
        .await?;
        Ok(())
    }

    async fn get(conn: &DatabaseConnection, id: &Uuid) -> Result<Option<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE id = ?", Self::table());

        let result = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![id.to_string().into()],
        )
        .all(conn)
        .await?;

        if result.is_empty() {
            return Ok(None);
        }

        let row = &result[0];
        Ok(Some(NodeLink::from_query_result(row, "")?))
    }
}

#[allow(dead_code)] // TODO: remove this
#[async_trait]
pub trait NodeExt {
    async fn get_by_project_id(conn: &DatabaseConnection, project_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl NodeExt for NodeLink {
    async fn get_by_project_id(conn: &DatabaseConnection, project_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE project_id = ?", Self::table());

        let results = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![project_id.to_string().into()],
        )
        .all(conn)
        .await?;

        results
            .iter()
            .map(|row| NodeLink::from_query_result(row, ""))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.into())
    }
}

#[allow(dead_code)] // TODO: remove this
#[async_trait]
trait NodeLinkExt {
    async fn get_by_node_id(conn: &DatabaseConnection, node_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl NodeLinkExt for NodeLink {
    async fn get_by_node_id(conn: &DatabaseConnection, node_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!(
            "SELECT id, project_id, left, right, linktype FROM {} WHERE left = ? OR right = ?",
            Self::table()
        );

        let results = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![node_id.to_string().into(), node_id.to_string().into()],
        )
        .all(conn)
        .await?;

        results
            .iter()
            .map(|row| NodeLink::from_query_result(row, ""))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.into())
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
        let conn = start_db(None).await.unwrap();
        NodeLink::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Node");
    }

    #[tokio::test]
    async fn test_crud() {
        let conn = start_db(None).await.unwrap();

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
            description: None,
            tags: Vec::new(),
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
