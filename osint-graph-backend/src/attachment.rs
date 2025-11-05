use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
use std::io::{Read, Write};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{entity::attachment, project::WebError, SharedState};

/// Upload a file attachment to a node
/// POST /api/v1/node/{id}/attachment
pub async fn upload_attachment(
    State(state): State<SharedState>,
    Path(node_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<Json<attachment::Model>, WebError> {
    let conn = &state.read().await.conn;

    debug!("Starting file upload for node {}", node_id);

    // Extract file from multipart form data
    let mut filename = None;
    let mut content_type = None;
    let mut data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {:?}", e);
        WebError::new(
            StatusCode::BAD_REQUEST,
            format!("Failed to read multipart field: {}", e),
        )
    })? {
        let field_name = field.name().unwrap_or("").to_string();
        debug!("Processing field: {}", field_name);

        match field_name.as_str() {
            "file" => {
                filename = field.file_name().map(|s| s.to_string());
                content_type = field.content_type().map(|s| s.to_string());
                debug!(
                    "File name: {:?}, content type: {:?}",
                    filename, content_type
                );

                data = Some(field.bytes().await.map_err(|e| {
                    error!("Failed to read file data: {:?}", e);
                    WebError::new(
                        StatusCode::BAD_REQUEST,
                        format!("Failed to read file data: {}", e),
                    )
                })?);

                debug!(
                    "Successfully read {} bytes",
                    data.as_ref().map(|d| d.len()).unwrap_or(0)
                );
            }
            _ => {
                debug!("Ignoring unknown multipart field: {}", field_name);
            }
        }
    }

    let filename = filename.ok_or_else(|| {
        WebError::new(
            StatusCode::BAD_REQUEST,
            "Missing filename in upload".to_string(),
        )
    })?;

    let content_type = content_type.unwrap_or_else(|| "application/octet-stream".to_string());

    let file_data = data
        .ok_or_else(|| {
            WebError::new(
                StatusCode::BAD_REQUEST,
                "Missing file data in upload".to_string(),
            )
        })?
        .to_vec();

    // Compress data with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&file_data).map_err(|e| {
        WebError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to compress attachment data: {}", e),
        )
    })?;
    let compressed_data = encoder.finish().map_err(|e| {
        WebError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to finish compression: {}", e),
        )
    })?;

    // Create attachment entity
    let attachment_id = Uuid::new_v4();
    let now = chrono::Utc::now();

    let new_attachment = attachment::ActiveModel {
        id: Set(attachment_id.to_string()),
        node_id: Set(node_id.to_string()),
        filename: Set(filename.clone()),
        content_type: Set(content_type.clone()),
        size: Set(file_data.len() as i64),
        data: Set(compressed_data),
        created: Set(now.to_rfc3339()),
    };

    // Save to database
    let saved = new_attachment.insert(conn).await.map_err(|e| {
        error!("Failed to save attachment: {:?}", e);
        WebError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save attachment: {}", e),
        )
    })?;

    debug!("Created attachment {} for node {}", attachment_id, node_id);

    Ok(Json(saved))
}

/// Download a file attachment
/// GET /api/v1/node/{node_id}/attachment/{attachment_id}
pub async fn download_attachment(
    State(state): State<SharedState>,
    Path((node_id, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<Response, WebError> {
    let conn = &state.read().await.conn;

    // Get attachment from database
    let attachment = attachment::Entity::find_by_id(attachment_id.to_string())
        .one(conn)
        .await
        .map_err(|e| {
            error!("Failed to get attachment: {:?}", e);
            WebError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get attachment: {}", e),
            )
        })?
        .ok_or_else(|| WebError::not_found(format!("Attachment {} not found", attachment_id)))?;

    // Verify attachment belongs to the node
    if attachment.node_id != node_id.to_string() {
        return Err(WebError::new(
            StatusCode::BAD_REQUEST,
            format!(
                "Attachment {} does not belong to node {}",
                attachment_id, node_id
            ),
        ));
    }

    // Decompress data
    let mut decoder = GzDecoder::new(&attachment.data[..]);
    let mut decompressed_data = Vec::new();
    decoder.read_to_end(&mut decompressed_data).map_err(|e| {
        WebError::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to decompress attachment data: {}", e),
        )
    })?;

    debug!(
        "Downloading attachment {} for node {}",
        attachment_id, node_id
    );

    // Return file with appropriate headers
    Ok((
        StatusCode::OK,
        [
            ("Content-Type", attachment.content_type.as_str()),
            (
                "Content-Disposition",
                &format!("attachment; filename=\"{}\"", attachment.filename),
            ),
        ],
        decompressed_data,
    )
        .into_response())
}

/// Delete a file attachment
/// DELETE /api/v1/node/{node_id}/attachment/{attachment_id}
pub async fn delete_attachment(
    State(state): State<SharedState>,
    Path((_node_id, attachment_id)): Path<(Uuid, Uuid)>,
) -> Result<String, WebError> {
    let conn = &state.read().await.conn;

    // Just attempt deletion, don't validate if it exists
    attachment::Entity::delete_by_id(attachment_id.to_string())
        .exec(conn)
        .await
        .map_err(|e| {
            error!("Failed to delete attachment: {:?}", e);
            WebError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to delete attachment: {}", e),
            )
        })?;

    debug!("Deleted attachment {}", attachment_id);

    Ok("Attachment deleted successfully".to_string())
}

/// List all attachments for a node
/// GET /api/v1/node/{node_id}/attachments
pub async fn list_attachments(
    State(state): State<SharedState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<Vec<attachment::Model>>, WebError> {
    let conn = &state.read().await.conn;

    let attachments = attachment::Entity::find()
        .filter(attachment::Column::NodeId.eq(node_id.to_string()))
        .all(conn)
        .await
        .map_err(|e| {
            error!("Failed to list attachments: {:?}", e);
            WebError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to list attachments: {:?}", e),
            )
        })?;

    debug!(
        "Listed {} attachments for node {}",
        attachments.len(),
        node_id
    );

    Ok(Json(attachments))
}
