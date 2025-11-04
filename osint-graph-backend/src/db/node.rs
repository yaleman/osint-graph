use axum::async_trait;
use osint_graph_shared::node::Node;

use sea_orm::{ConnectionTrait, DatabaseConnection, FromQueryResult};
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Node {
    fn table() -> &'static str {
        "node"
    }

    async fn create_table(_conn: &DatabaseConnection) -> Result<(), DBError> {
        // Tables are now created via migrations
        debug!("Skipping create table - using migrations");
        Ok(())
    }

    async fn save(&self, conn: &DatabaseConnection) -> Result<(), DBError> {
        let attachments_encoded = serde_json::to_string(&self.attachments)?;

        let querystring = format!(
            "INSERT INTO {} (id, project_id, node_type, display, value, updated, notes, pos_x, pos_y, attachments) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT(id) DO
                UPDATE SET project_id = ?, node_type = ?, display = ?, value = ?, updated = ?, notes = ?, pos_x = ?, pos_y = ?, attachments = ?;",
            Self::table()
        );

        let _ = conn
            .execute(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
                vec![
                    self.id.to_string().into(),
                    self.project_id.to_string().into(),
                    self.node_type.clone().into(),
                    self.display.clone().into(),
                    self.value.clone().into(),
                    self.updated.to_rfc3339().into(),
                    self.notes.clone().into(),
                    self.pos_x.into(),
                    self.pos_y.into(),
                    attachments_encoded.clone().into(),
                    self.project_id.to_string().into(),
                    self.node_type.clone().into(),
                    self.display.clone().into(),
                    self.value.clone().into(),
                    self.updated.to_rfc3339().into(),
                    self.notes.clone().into(),
                    self.pos_x.into(),
                    self.pos_y.into(),
                    attachments_encoded.into(),
                ],
            ))
            .await?;
        Ok(())
    }

    async fn get(conn: &DatabaseConnection, id: &Uuid) -> Result<Option<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE id = ?", Self::table());

        let result = conn
            .query_all(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
                vec![id.to_string().into()],
            ))
            .await?;

        if result.is_empty() {
            return Ok(None);
        }

        let row = &result[0];
        Ok(Some(Node::from_query_result(row, "")?))
    }
}

#[async_trait]
pub trait NodeExt {
    async fn get_by_project_id(conn: &DatabaseConnection, project_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl NodeExt for Node {
    async fn get_by_project_id(conn: &DatabaseConnection, project_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE project_id = ?", Self::table());

        let results = conn
            .query_all(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
                vec![project_id.to_string().into()],
            ))
            .await?;

        results
            .iter()
            .map(|row| Node::from_query_result(row, ""))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.into())
    }
}

#[cfg(test)]
mod tests {

    use chrono::DateTime;
    use osint_graph_shared::node::NodeUpdateList;
    use osint_graph_shared::project::Project;

    use crate::storage::start_db;

    use super::*;

    #[tokio::test]
    async fn test_create_table() {
        let pool = start_db(None).await.unwrap();
        Node::create_table(&pool)
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

        let mut node = Node {
            project_id,
            id,
            node_type: "person".to_string(),
            display: "Test Person".to_string(),
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
