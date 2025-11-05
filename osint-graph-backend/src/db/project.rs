use sea_orm::prelude::async_trait;
use osint_graph_shared::project::Project;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Project {
    fn table() -> &'static str {
        "project"
    }

    async fn create_table(_conn: &DatabaseConnection) -> Result<(), DBError> {
        // Tables are now created via migrations
        debug!("Skipping create table - using migrations");
        Ok(())
    }

    async fn save(&self, conn: &DatabaseConnection) -> Result<(), DBError> {
        let querystring = format!(
            "INSERT INTO {} (id, name, user, creationdate, last_updated, nodes, description, tags ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO
            UPDATE SET name = ?, user = ?, creationdate = ?, last_updated = ?, nodes = ?, description = ?, tags = ? ",
            Self::table()
        );
        let nodes_encoded = serde_json::to_string(&self.nodes)?;
        let tags_encoded = serde_json::to_string(&self.tags)?;

        let _ = conn
            .execute(sea_orm::Statement::from_sql_and_values(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
                vec![
                    self.id.to_string().into(),
                    self.name.clone().into(),
                    self.user.to_string().into(),
                    self.creationdate.to_rfc3339().into(),
                    self.last_updated.map(|d| d.to_rfc3339()).into(),
                    nodes_encoded.clone().into(),
                    self.description.clone().into(),
                    tags_encoded.clone().into(),
                    self.name.clone().into(),
                    self.user.to_string().into(),
                    self.creationdate.to_rfc3339().into(),
                    self.last_updated.map(|d| d.to_rfc3339()).into(),
                    nodes_encoded.into(),
                    self.description.clone().into(),
                    tags_encoded.into(),
                ],
            ))
            .await?;

        Ok(())
    }

    async fn get(conn: &DatabaseConnection, id: &Uuid) -> Result<Option<Self>, DBError> {
        get_project_by_id(conn, id).await
    }
}

// Helper function since we can't add impl blocks to types from other crates
async fn get_project_by_id(conn: &DatabaseConnection, id: &Uuid) -> Result<Option<Project>, DBError> {
    use sea_orm::FromQueryResult;

    let querystring = format!("SELECT * FROM {} WHERE id = ?", Project::table());

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
    Ok(Some(Project::from_query_result(row, "")?))
}

#[async_trait]
pub trait DBProjectExt {
    async fn get_all(conn: &DatabaseConnection) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl DBProjectExt for Project {
    async fn get_all(conn: &DatabaseConnection) -> Result<Vec<Self>, DBError> {
        use sea_orm::FromQueryResult;

        let querystring = format!("SELECT * FROM {}", Project::table());

        let results = conn
            .query_all(sea_orm::Statement::from_string(
                sea_orm::DatabaseBackend::Sqlite,
                querystring,
            ))
            .await?;

        results
            .iter()
            .map(|row| Project::from_query_result(row, ""))
            .collect::<Result<Vec<_>, _>>()
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
        let conn = start_db(None).await.unwrap();
        Project::create_table(&conn)
            .await
            .expect("Failed to create in-memory table for Project");
    }

    #[tokio::test]
    async fn test_crud() {
        let conn = start_db(None).await.unwrap();

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
