use axum::async_trait;
use chrono::{DateTime, Utc};
use osint_graph_shared::attachment::Attachment;
use sea_orm::{DatabaseConnection, FromQueryResult};
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Attachment {
    fn table() -> &'static str {
        "attachment"
    }

    async fn create_table(_conn: &DatabaseConnection) -> Result<(), DBError> {
        // Tables are now created via migrations
        debug!("Skipping create table - using migrations");
        Ok(())
    }

    async fn save(&self, conn: &DatabaseConnection) -> Result<(), DBError> {
        // Compress data with zstd before saving
        let compressed_data = zstd::encode_all(&self.data[..], 3)
            .map_err(|e| DBError::Other(format!("Failed to compress attachment data: {}", e)))?;

        let querystring = format!(
            "INSERT INTO {} (id, node_id, filename, content_type, size, data, created)
             VALUES (?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(id) DO
             UPDATE SET node_id = ?, filename = ?, content_type = ?, size = ?, data = ?, created = ?;",
            Self::table()
        );

        sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![
                self.id.to_string().into(),
                self.node_id.to_string().into(),
                self.filename.clone().into(),
                self.content_type.clone().into(),
                self.size.into(),
                compressed_data.clone().into(),
                self.created.to_rfc3339().into(),
                self.node_id.to_string().into(),
                self.filename.clone().into(),
                self.content_type.clone().into(),
                self.size.into(),
                compressed_data.into(),
                self.created.to_rfc3339().into(),
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
        let attachment_row = AttachmentRow::from_query_result(row, "")?;

        // Decompress data when loading
        let decompressed_data = zstd::decode_all(&attachment_row.data[..]).map_err(|e| {
            DBError::Other(format!("Failed to decompress attachment data: {}", e))
        })?;

        Ok(Some(Attachment {
            id: attachment_row.id,
            node_id: attachment_row.node_id,
            filename: attachment_row.filename,
            content_type: attachment_row.content_type,
            size: attachment_row.size,
            data: decompressed_data,
            created: attachment_row.created,
        }))
    }
}

// Helper struct for deserializing from database (with compressed data)
#[derive(FromQueryResult)]
struct AttachmentRow {
    pub id: Uuid,
    pub node_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub data: Vec<u8>,
    pub created: DateTime<Utc>,
}

/// Extension trait for attachment-specific operations
#[async_trait]
pub trait AttachmentExt {
    /// Get all attachments for a specific node
    async fn get_by_node_id(conn: &DatabaseConnection, node_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl AttachmentExt for Attachment {
    async fn get_by_node_id(conn: &DatabaseConnection, node_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE node_id = ?", Self::table());

        let results = sea_orm::Statement::from_sql_and_values(
            sea_orm::DatabaseBackend::Sqlite,
            querystring,
            vec![node_id.to_string().into()],
        )
        .all(conn)
        .await?;

        // Convert rows and decompress all attachments
        results
            .iter()
            .map(|row| {
                let attachment_row = AttachmentRow::from_query_result(row, "")?;

                let decompressed_data = zstd::decode_all(&attachment_row.data[..]).map_err(|e| {
                    DBError::Other(format!("Failed to decompress attachment data: {}", e))
                })?;

                Ok(Attachment {
                    id: attachment_row.id,
                    node_id: attachment_row.node_id,
                    filename: attachment_row.filename,
                    content_type: attachment_row.content_type,
                    size: attachment_row.size,
                    data: decompressed_data,
                    created: attachment_row.created,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::start_db;
    use osint_graph_shared::node::Node;
    use osint_graph_shared::project::Project;

    #[tokio::test]
    async fn test_create_table() {
        let conn = start_db(None).await.unwrap();
        Attachment::create_table(&conn)
            .await
            .expect("Failed to create attachment table");
    }

    #[tokio::test]
    async fn test_attachment_crud_with_compression() {
        let conn = start_db(None).await.unwrap();

        // Create project and node first
        let project_id = Uuid::new_v4();
        let node_id = Uuid::new_v4();

        let project = Project {
            id: project_id,
            name: "Test Project".to_string(),
            user: Uuid::new_v4(),
            creationdate: chrono::Utc::now(),
            last_updated: None,
            nodes: Default::default(),
            description: None,
            tags: Vec::new(),
        };
        project.save(&conn).await.expect("Failed to save project");

        let node = Node {
            project_id,
            id: node_id,
            node_type: "document".to_string(),
            display: "Test Document".to_string(),
            value: "test.pdf".to_string(),
            updated: chrono::Utc::now(),
            notes: None,
            pos_x: None,
            pos_y: None,
            attachments: Vec::new(),
        };
        node.save(&conn).await.expect("Failed to save node");

        // Create test attachment with some data
        let test_data = b"This is test file content that will be compressed".to_vec();
        let attachment = Attachment::new(
            node_id,
            "test.txt".to_string(),
            "text/plain".to_string(),
            test_data.clone(),
        );

        // Save attachment (should compress data)
        attachment
            .save(&conn)
            .await
            .expect("Failed to save attachment");

        // Retrieve attachment (should decompress data)
        let retrieved = Attachment::get(&conn, &attachment.id)
            .await
            .expect("Failed to get attachment")
            .expect("Attachment not found");

        // Verify data matches original
        assert_eq!(retrieved.id, attachment.id);
        assert_eq!(retrieved.node_id, node_id);
        assert_eq!(retrieved.filename, "test.txt");
        assert_eq!(retrieved.content_type, "text/plain");
        assert_eq!(retrieved.size, test_data.len() as i64);
        assert_eq!(retrieved.data, test_data);

        // Test get_by_node_id
        let attachments = Attachment::get_by_node_id(&conn, node_id)
            .await
            .expect("Failed to get attachments by node_id");
        assert_eq!(attachments.len(), 1);
        assert_eq!(attachments[0].data, test_data);

        // Test deletion
        Attachment::delete_by_id(&conn, attachment.id)
            .await
            .expect("Failed to delete attachment");

        let deleted = Attachment::get(&conn, &attachment.id)
            .await
            .expect("Failed to query for deleted attachment");
        assert!(deleted.is_none());
    }

    #[tokio::test]
    async fn test_cascade_deletion() {
        let conn = start_db(None).await.unwrap();

        // Create project, node, and attachment
        let project_id = Uuid::new_v4();
        let node_id = Uuid::new_v4();

        let project = Project {
            id: project_id,
            name: "Test Project".to_string(),
            user: Uuid::new_v4(),
            creationdate: chrono::Utc::now(),
            last_updated: None,
            nodes: Default::default(),
            description: None,
            tags: Vec::new(),
        };
        project.save(&conn).await.expect("Failed to save project");

        let node = Node {
            project_id,
            id: node_id,
            node_type: "document".to_string(),
            display: "Test Document".to_string(),
            value: "test.pdf".to_string(),
            updated: chrono::Utc::now(),
            notes: None,
            pos_x: None,
            pos_y: None,
            attachments: Vec::new(),
        };
        node.save(&conn).await.expect("Failed to save node");

        let test_data = b"Test file for cascade deletion".to_vec();
        let attachment = Attachment::new(
            node_id,
            "cascade_test.txt".to_string(),
            "text/plain".to_string(),
            test_data,
        );
        attachment
            .save(&conn)
            .await
            .expect("Failed to save attachment");

        // Verify attachment exists
        let found = Attachment::get(&conn, &attachment.id)
            .await
            .expect("Failed to get attachment");
        assert!(found.is_some());

        // Delete the node
        Node::delete_by_id(&conn, node_id)
            .await
            .expect("Failed to delete node");

        // Verify attachment was cascade deleted
        let deleted = Attachment::get(&conn, &attachment.id)
            .await
            .expect("Failed to query attachment");
        assert!(deleted.is_none());
    }
}
