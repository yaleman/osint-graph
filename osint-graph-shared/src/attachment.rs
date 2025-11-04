use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, FromRow};
use uuid::Uuid;

/// Represents a file attachment associated with a node
#[derive(Encode, Decode, FromRow, Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub struct Attachment {
    /// Unique identifier for this attachment
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,

    /// ID of the node this attachment belongs to
    pub node_id: Uuid,

    /// Original filename of the attachment
    pub filename: String,

    /// MIME type of the file (e.g., "image/png", "application/pdf")
    pub content_type: String,

    /// Size of the file in bytes (uncompressed)
    pub size: i64,

    /// File data, stored as zstd-compressed bytes
    #[sqlx(default)]
    pub data: Vec<u8>,

    /// When this attachment was created
    pub created: DateTime<Utc>,
}

impl Attachment {
    /// Create a new attachment with the given data
    pub fn new(node_id: Uuid, filename: String, content_type: String, data: Vec<u8>) -> Self {
        let size = data.len() as i64;
        Self {
            id: Uuid::new_v4(),
            node_id,
            filename,
            content_type,
            size,
            data,
            created: Utc::now(),
        }
    }
}
