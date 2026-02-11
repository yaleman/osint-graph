use axum::{
    body::Body,
    extract::{
        multipart::{MultipartError, MultipartRejection},
        Multipart, Path, State,
    },
    http::{
        header::{ACCEPT_ENCODING, CONTENT_DISPOSITION, CONTENT_ENCODING, CONTENT_TYPE, COOKIE},
        HeaderMap, HeaderValue, StatusCode,
    },
    response::Response,
    Json,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use futures::stream;
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, IntoActiveModel, QueryFilter,
    TryIntoModel,
};
use serde::Deserialize;
use std::io::{Cursor, Read, Write};
use tokio::sync::mpsc;
use tracing::{debug, error};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    entity::{attachment, node},
    project::WebError,
    SharedState,
};

const ATTACHMENT_STREAM_CHUNK_SIZE: usize = 16 * 1024;
pub const MAX_ATTACHMENT_UPLOAD_MB: usize = 100;
pub const MAX_ATTACHMENT_UPLOAD_BYTES: usize = MAX_ATTACHMENT_UPLOAD_MB * 1024 * 1024;
pub const MAX_ATTACHMENT_MULTIPART_REQUEST_BYTES: usize =
    MAX_ATTACHMENT_UPLOAD_BYTES + (1024 * 1024);
pub const ATTACHMENT_TOO_LARGE_ERROR: &str = "Attachment exceeds maximum size of 100 MB";

fn attachment_too_large_error() -> WebError {
    WebError::new(StatusCode::PAYLOAD_TOO_LARGE, ATTACHMENT_TOO_LARGE_ERROR)
}

fn map_multipart_rejection(rejection: MultipartRejection) -> WebError {
    if rejection.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return attachment_too_large_error();
    }

    WebError::new(
        StatusCode::BAD_REQUEST,
        format!("Invalid multipart upload: {}", rejection.body_text()),
    )
}

fn map_multipart_error(context: &str, error: MultipartError) -> WebError {
    if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
        return attachment_too_large_error();
    }

    WebError::new(
        StatusCode::BAD_REQUEST,
        format!("{}: {}", context, error.body_text()),
    )
}

fn stream_gzip_decompressed_body(compressed_data: Vec<u8>) -> Result<Body, WebError> {
    // Preflight read so malformed gzip payloads still return a 500 before response streaming starts.
    let mut decoder = GzDecoder::new(Cursor::new(compressed_data));
    let mut first_chunk = vec![0; ATTACHMENT_STREAM_CHUNK_SIZE];
    let first_read = decoder.read(&mut first_chunk).map_err(|e| {
        WebError::internal_server_error(format!("Failed to decompress attachment data: {}", e))
    })?;
    first_chunk.truncate(first_read);

    let (tx, rx) = mpsc::channel::<Result<Vec<u8>, std::io::Error>>(8);
    tokio::task::spawn_blocking(move || {
        if !first_chunk.is_empty() && tx.blocking_send(Ok(first_chunk)).is_err() {
            return;
        }

        let mut chunk = vec![0; ATTACHMENT_STREAM_CHUNK_SIZE];
        loop {
            match decoder.read(&mut chunk) {
                Ok(0) => break,
                Ok(bytes_read) => {
                    if tx.blocking_send(Ok(chunk[..bytes_read].to_vec())).is_err() {
                        break;
                    }
                }
                Err(err) => {
                    let _ = tx.blocking_send(Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Failed to decompress attachment data: {}", err),
                    )));
                    break;
                }
            }
        }
    });

    let body_stream = stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|item| (item, rx))
    });
    Ok(Body::from_stream(body_stream))
}

/// Upload a file attachment to a node
#[utoipa::path(
    post,
    path = "/api/v1/node/{id}/attachment",
    responses(
        (status = OK, description = "Attachment uploaded successfully", body = attachment::Model),
        (status = BAD_REQUEST, description = "Invalid request"),
        (status = PAYLOAD_TOO_LARGE, description = "Attachment exceeds maximum size"),
        (status = NOT_FOUND, description = "Node not found")
    )
)]
pub async fn upload_attachment(
    State(state): State<SharedState>,
    Path(node_id): Path<Uuid>,
    multipart: Result<Multipart, MultipartRejection>,
) -> Result<Json<attachment::Model>, WebError> {
    let conn = &state.read().await.conn;
    let mut multipart = multipart.map_err(map_multipart_rejection)?;

    debug!("Starting file upload for node {}", node_id);

    // Extract file from multipart form data
    let mut filename = None;
    let mut content_type = None;
    let mut data = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Failed to read multipart field: {:?}", e);
        map_multipart_error("Failed to read multipart field", e)
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
                    map_multipart_error("Failed to read file data", e)
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

    if file_data.len() > MAX_ATTACHMENT_UPLOAD_BYTES {
        return Err(attachment_too_large_error());
    }

    // Verify the node exists before creating the attachment
    let node_exists = node::Entity::find_by_id(node_id)
        .one(conn)
        .await
        .map_err(|e| {
            error!("Failed to check if node exists: {:?}", e);
            WebError::internal_server_error(format!("Failed to verify node: {}", e))
        })?
        .is_some();

    if !node_exists {
        return Err(WebError::not_found(format!("Node {} not found", node_id)));
    }

    // Compress data with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&file_data).map_err(|e| {
        WebError::internal_server_error(format!("Failed to compress attachment data: {}", e))
    })?;
    let compressed_data = encoder.finish().map_err(|e| {
        WebError::internal_server_error(format!("Failed to finish compression: {}", e))
    })?;

    // Create attachment entity

    let new_attachment = attachment::ActiveModel {
        id: Set(Uuid::new_v4()),
        node_id: Set(node_id),
        filename: Set(filename),
        content_type: Set(content_type.clone()),
        size: Set(file_data.len() as i64),
        data: Set(compressed_data),
        created: Set(chrono::Utc::now()),
    };

    // Save to database
    let saved = new_attachment.insert(conn).await.map_err(|e| {
        error!("Failed to save attachment: {:?}", e);
        WebError::internal_server_error(format!("Failed to save attachment: {}", e))
    })?;

    debug!(
        attachment_id = saved.id.to_string(),
        node_id = node_id.to_string(),
        "Created attachment"
    );

    Ok(Json(saved))
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct UpdateAttachmentData {
    node_id: Option<Uuid>,
    data: Option<Vec<u8>>,
}

/// Update a file attachment's metadata or data
#[utoipa::path(
    put,
    path = "/api/v1/attachment/{attachment_id}",
    request_body = UpdateAttachmentData,
    responses(
        (status = OK, description = "Attachment updated successfully", body = attachment::Model),
        (status = NOT_FOUND, description = "Attachment not found"),
        (status = BAD_REQUEST, description = "Invalid request")
    )
)]
pub async fn update_attachment(
    State(state): State<SharedState>,
    Path(attachment_id): Path<Uuid>,
    Json(update_data): Json<UpdateAttachmentData>,
) -> Result<Json<attachment::Model>, WebError> {
    let conn = &state.read().await.conn;

    // Find the attachment
    let attachment = attachment::Entity::find_by_id(attachment_id)
        .one(conn)
        .await
        .map_err(|e| {
            error!("Failed to get attachment: {:?}", e);
            WebError::internal_server_error(format!("Failed to get attachment: {}", e))
        })?
        .ok_or_else(|| WebError::not_found(format!("Attachment {} not found", attachment_id)))?;

    // Update the attachment
    let mut updated_attachment = attachment.into_active_model();
    if let Some(node_id) = update_data.node_id {
        updated_attachment.node_id = Set(node_id);
    }
    if let Some(data) = update_data.data {
        updated_attachment.data = Set(data);
    }

    if updated_attachment.is_changed() {
        debug!(
            attachment_id = attachment_id.to_string(),
            "Updating attachment"
        );
        // Save the updated attachment
        let updated_attachment = updated_attachment.update(conn).await.map_err(|e| {
            error!("Failed to update attachment: {:?}", e);
            WebError::internal_server_error(format!("Failed to update attachment: {}", e))
        })?;
        Ok(Json(updated_attachment))
    } else {
        debug!(
            attachment_id = attachment_id.to_string(),
            "No changes to update for attachment"
        );
        Ok(Json(updated_attachment.try_into_model()?))
    }
}

/// Download a file attachment
#[utoipa::path(
    get,
    path = "/api/v1/attachment/{attachment_id}",
    responses(
        (status = OK, description = "Attachment downloaded successfully", content_type = "application/octet-stream", body = [u8]),
        (status = NOT_FOUND, description = "Attachment not found"),
        (status = BAD_REQUEST, description = "Attachment does not belong to node")
    )
)]
pub async fn download_attachment(
    State(state): State<SharedState>,
    Path(attachment_id): Path<Uuid>,
) -> Result<Response, WebError> {
    let conn = &state.read().await.conn;

    // Get attachment from database
    let attachment = attachment::Entity::find_by_id(attachment_id)
        .one(conn)
        .await
        .map_err(|e| {
            error!("Failed to get attachment: {:?}", e);
            WebError::internal_server_error(format!("Failed to get attachment: {}", e))
        })?
        .ok_or_else(|| WebError::not_found(format!("Attachment {} not found", attachment_id)))?;

    let body = stream_gzip_decompressed_body(attachment.data)?;

    debug!(
        attachment_id = attachment_id.to_string(),
        node_id = attachment.node_id.to_string(),
        "Downloading attachment",
    );

    // Return file with appropriate headers
    let mut res = Response::new(body);
    *res.status_mut() = StatusCode::OK;
    res.headers_mut().insert(
        CONTENT_TYPE,
        HeaderValue::from_str(attachment.content_type.as_str())?,
    );
    res.headers_mut().insert(
        CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!("attachment; filename=\"{}\"", attachment.filename))?,
    );

    Ok(res)
}

/// View a file attachment (inline display for images, PDFs, text)
/// GET /api/v1//attachment/{attachment_id}/view
#[utoipa::path(
    get,
    path = "/api/v1/attachment/{attachment_id}/view",
    responses(
        (status = OK, description = "Attachment retrieved successfully", content_type = "application/octet-stream", body = [u8]),
        (status = NOT_FOUND, description = "Attachment not found")
    )
)]
pub async fn view_attachment(
    headers: HeaderMap,
    State(state): State<SharedState>,
    Path(attachment_id): Path<Uuid>,
) -> Result<Response, WebError> {
    // Get attachment from database
    let attachment = attachment::Entity::find_by_id(attachment_id)
        .one(&state.read().await.conn)
        .await
        .map_err(|e| {
            error!("Failed to get attachment: {:?}", e);
            WebError::internal_server_error(format!("Failed to get attachment: {}", e))
        })?
        .ok_or_else(|| WebError::not_found(format!("Attachment {} not found", attachment_id)))?;

    let mut need_decompress = false;

    if let Some(accept) = headers.get(ACCEPT_ENCODING) {
        if accept.to_str().unwrap_or("").contains("gzip") {
            need_decompress = true;
        }
    }

    debug!(
        attachment_id = attachment_id.to_string(),
        node_id = attachment.node_id.to_string(),
        requires_decompression = need_decompress,
        "Viewing attachment"
    );

    let headers = [
        (
            CONTENT_TYPE,
            HeaderValue::from_str(attachment.content_type.as_str())?,
        ),
        (
            CONTENT_DISPOSITION,
            HeaderValue::from_str(&format!("inline; filename=\"{}\"", attachment.filename))?,
        ),
        (COOKIE, HeaderValue::from_static("")),
    ];
    if need_decompress {
        let body = stream_gzip_decompressed_body(attachment.data)?;
        let mut res = Response::new(body);
        *res.status_mut() = StatusCode::OK;
        res.headers_mut().extend(headers);
        Ok(res)
    } else {
        let mut headers_vec = headers.to_vec();
        headers_vec.push((CONTENT_ENCODING, HeaderValue::from_static("gzip")));
        // Return file with inline disposition for viewing in browser
        let mut res = Response::new(Body::from(attachment.data));
        *res.status_mut() = StatusCode::OK;
        res.headers_mut().extend(headers_vec);
        Ok(res)
    }
}

/// Delete a file attachment
#[utoipa::path(
    delete,
    path = "/api/v1/attachment/{attachment_id}",
    responses(
        (status = OK, description = "Attachment deleted successfully", body = String)
    )
)]
pub async fn delete_attachment(
    State(state): State<SharedState>,
    Path(attachment_id): Path<Uuid>,
) -> Result<String, WebError> {
    // Just attempt deletion, don't validate if it exists
    match attachment::Entity::delete_by_id(attachment_id)
        .exec(&state.read().await.conn)
        .await
        .map_err(|e| {
            error!("Failed to delete attachment: {:?}", e);
            WebError::internal_server_error(format!("Failed to delete attachment: {}", e))
        })?
        .rows_affected
    {
        0 => Err(WebError::not_found(format!(
            "Attachment {} not found",
            attachment_id
        ))),
        _ => Ok("Attachment deleted successfully".to_string()),
    }
}

/// List all attachments for a node, does not include file data
#[utoipa::path(
    get,
    path = "/api/v1/node/{id}/attachments",
    responses(
        (status = OK, description = "Attachments retrieved successfully", body = Vec<attachment::Model>)
    )
)]
pub async fn list_attachments(
    State(state): State<SharedState>,
    Path(node_id): Path<Uuid>,
) -> Result<Json<Vec<attachment::Model>>, WebError> {
    let attachments = attachment::Entity::find()
        .filter(attachment::Column::NodeId.eq(node_id))
        .all(&state.read().await.conn)
        .await
        .map_err(|e| {
            error!("Failed to list attachments: {:?}", e);
            WebError::internal_server_error(format!("Failed to list attachments: {:?}", e))
        })?
        .into_iter()
        .map(|mut a| {
            // Hide the data field when listing attachments
            a.data = Vec::new();
            a
        })
        .collect::<Vec<_>>();

    debug!(
        "Listed {} attachments for node {}",
        attachments.len(),
        node_id
    );

    Ok(Json(attachments))
}
