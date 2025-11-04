use axum::async_trait;
use osint_graph_shared::attachment::Attachment;
use sqlx::SqlitePool;
use tracing::debug;
use uuid::Uuid;

use crate::storage::{DBEntity, DBError};

#[async_trait]
impl DBEntity for Attachment {
    fn table() -> &'static str {
        "attachment"
    }

    async fn create_table(pool: &SqlitePool) -> Result<(), DBError> {
        sqlx::query(&format!(
            "
            CREATE TABLE IF NOT EXISTS {} (
                id TEXT PRIMARY KEY,
                node_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                content_type TEXT NOT NULL,
                size INTEGER NOT NULL,
                data BLOB NOT NULL,
                created TEXT NOT NULL,
                FOREIGN KEY (node_id) REFERENCES node(id) ON DELETE CASCADE ON UPDATE CASCADE
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

        sqlx::query(&querystring)
            .bind(self.id)
            .bind(self.node_id)
            .bind(self.filename.clone())
            .bind(self.content_type.clone())
            .bind(self.size)
            .bind(&compressed_data)
            .bind(self.created)
            .bind(self.node_id)
            .bind(self.filename.clone())
            .bind(self.content_type.clone())
            .bind(self.size)
            .bind(&compressed_data)
            .bind(self.created)
            .execute(pool)
            .await?;

        Ok(())
    }

    async fn get(pool: &SqlitePool, id: &Uuid) -> Result<Option<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE id = ?", Self::table());

        let result: Option<AttachmentRow> = sqlx::query_as(&querystring)
            .bind(id)
            .fetch_optional(pool)
            .await?;

        match result {
            Some(row) => {
                // Decompress data when loading
                let decompressed_data = zstd::decode_all(&row.data[..]).map_err(|e| {
                    DBError::Other(format!("Failed to decompress attachment data: {}", e))
                })?;

                Ok(Some(Attachment {
                    id: row.id,
                    node_id: row.node_id,
                    filename: row.filename,
                    content_type: row.content_type,
                    size: row.size,
                    data: decompressed_data,
                    created: row.created,
                }))
            }
            None => Ok(None),
        }
    }
}

// Helper struct for deserializing from database (with compressed data)
#[derive(sqlx::FromRow)]
struct AttachmentRow {
    pub id: Uuid,
    pub node_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub data: Vec<u8>,
    pub created: sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>,
}

/// Extension trait for attachment-specific operations
#[async_trait]
pub trait AttachmentExt {
    /// Get all attachments for a specific node
    async fn get_by_node_id(pool: &SqlitePool, node_id: Uuid) -> Result<Vec<Self>, DBError>
    where
        Self: Sized;
}

#[async_trait]
impl AttachmentExt for Attachment {
    async fn get_by_node_id(pool: &SqlitePool, node_id: Uuid) -> Result<Vec<Self>, DBError> {
        let querystring = format!("SELECT * FROM {} WHERE node_id = ?", Self::table());

        let rows: Vec<AttachmentRow> = sqlx::query_as(&querystring)
            .bind(node_id)
            .fetch_all(pool)
            .await?;

        // Decompress all attachments
        rows.into_iter()
            .map(|row| {
                let decompressed_data = zstd::decode_all(&row.data[..]).map_err(|e| {
                    DBError::Other(format!("Failed to decompress attachment data: {}", e))
                })?;

                Ok(Attachment {
                    id: row.id,
                    node_id: row.node_id,
                    filename: row.filename,
                    content_type: row.content_type,
                    size: row.size,
                    data: decompressed_data,
                    created: row.created,
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
        let conn = start_db(None, None).await.unwrap();
        Attachment::create_table(&conn)
            .await
            .expect("Failed to create attachment table");
    }

    #[tokio::test]
    async fn test_attachment_crud_with_compression() {
        let conn = start_db(None, None).await.unwrap();
        Attachment::create_table(&conn)
            .await
            .expect("Failed to create attachment table");

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
        let conn = start_db(None, None).await.unwrap();
        Attachment::create_table(&conn)
            .await
            .expect("Failed to create attachment table");

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
