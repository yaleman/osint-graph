use axum::extract::{Path, Query, State};
use axum::http::header::{InvalidHeaderValue, CONTENT_TYPE};
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use osint_graph_shared::node::NodeType;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    TransactionTrait, TryIntoModel,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use tracing::{debug, error, info};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::entity::{attachment, node, nodelink, project};
use crate::SharedState;

/// Clean URL values by removing invisible Unicode characters
/// Removes zero-width spaces, directional isolates, and other invisible formatting characters
fn clean_url_value(value: &str) -> String {
    value
        .trim()
        .chars()
        .filter(|c| {
            // Remove invisible Unicode characters that can break URLs
            !matches!(
                *c,
                '\u{200B}'
                    ..='\u{200D}' | // Zero-width spaces
                '\u{FEFF}' |               // Zero-width no-break space
                '\u{2069}' // Pop directional isolate
            )
        })
        .collect()
}

/// POST handler for project things
///
#[utoipa::path(
    post,
    path = "/api/v1/project",
    request_body = project::Model,
    responses(
        (status = OK, description = "Created a project", body = project::Model)
    )
)]
pub async fn post_project(
    State(state): State<SharedState>,
    Json(project): Json<project::Model>,
) -> Result<Json<project::Model>, WebError> {
    let project = match project::Entity::find_by_id(project.id)
        .one(&state.read().await.conn)
        .await?
    {
        Some(val) => {
            let mut target_project = val.into_active_model();
            target_project.description = Set(project.description);
            target_project.name = Set(project.name);
            target_project.tags = Set(project.tags.clone());
            target_project.last_updated = Set(Some(Utc::now()));

            target_project
                .update(&state.read().await.conn)
                .await
                .inspect_err(|err| error!("Failed to update project: {:?}", err))?
                .try_into_model()?
        }
        None => {
            let project = project.into_active_model();
            debug!("Creating project: {:?}", project);
            project
                .insert(&state.read().await.conn)
                .await
                .inspect_err(|err| error!("Failed to save project: {:?}", err))?
        }
    };

    Ok(Json(project))
}

pub struct WebError {
    status: StatusCode,
    message: String,
}

impl WebError {
    pub fn new(status: StatusCode, message: impl ToString) -> Self {
        WebError {
            status,
            message: message.to_string(),
        }
    }

    pub fn not_found(message: impl ToString) -> Self {
        WebError {
            status: StatusCode::NOT_FOUND,
            message: message.to_string(),
        }
    }

    pub fn internal_server_error(message: impl ToString) -> Self {
        WebError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.to_string(),
        }
    }
}

impl From<InvalidHeaderValue> for WebError {
    fn from(err: InvalidHeaderValue) -> Self {
        WebError::internal_server_error(format!("Invalid header value: {:?}", err))
    }
}

impl IntoResponse for WebError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": self.message,
        });
        let mut response = axum::response::Response::new(body.to_string().into());
        *response.status_mut() = self.status;
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        response
    }
}

impl From<DbErr> for WebError {
    fn from(err: DbErr) -> Self {
        WebError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Database error: {:?}", err),
        }
    }
}

impl From<serde_json::Error> for WebError {
    fn from(err: serde_json::Error) -> Self {
        WebError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: format!("Serialization error: {:?}", err),
        }
    }
}

/// Pulls a project from storage.

#[utoipa::path(
    get,
    path = "/api/v1/project/{id}",
    responses(
        (status = OK, description = "One result ok", body = project::Model)
    )
)]
pub async fn get_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<project::Model>, WebError> {
    let res = project::Entity::find_by_id(id)
        .one(&state.read().await.conn)
        .await?;

    match res {
        Some(project) => Ok(Json(project)),
        None => Err(WebError::not_found(format!("Project {} not found", id))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    responses(
        (status = OK, description = "One result ok", body = Vec<project::Model>)
    )
)]
pub async fn get_projects(
    State(state): State<SharedState>,
) -> Result<Json<Vec<project::Model>>, WebError> {
    let val = project::Entity::find()
        .all(&state.read().await.conn)
        .await
        .inspect_err(|err| error!(error=?err, "Failed to query project list"))?;
    Ok(Json(val))
}

#[utoipa::path(
    get,
    path = "/api/v1/node/{id}",
    responses(
        (status = OK, description = "One result ok", body = node::Model)
    )
)]
pub async fn get_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<node::Model>, WebError> {
    match node::Entity::find_by_id(id)
        .one(&state.read().await.conn)
        .await?
    {
        Some(val) => Ok(Json(val)),
        None => Err(WebError::not_found(format!("Node {} not found", id))),
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/project/{project_id}/nodes",
    responses(
        (status = OK, description = "One result ok", body = Vec<node::Model>)
    )
)]
pub async fn get_nodes_by_project(
    Path(project_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<Vec<node::Model>>, WebError> {
    let nodes = node::Entity::find()
        .filter(node::Column::ProjectId.eq(project_id))
        .all(&state.read().await.conn)
        .await
        .inspect_err(|err| error!("Failed to get nodes for project {}: {:?}", project_id, err))?;
    Ok(Json(nodes))
}

#[utoipa::path(
    post,
    path = "/api/v1/node",
    request_body = node::Model,
    responses(
        (status = OK, description = "One result ok", body = node::Model)
    )
)]
pub async fn post_node(
    State(state): State<SharedState>,
    Json(mut node): Json<node::Model>,
) -> Result<Json<node::Model>, WebError> {
    let txn = state
        .read()
        .await
        .conn
        .begin()
        .await
        .inspect_err(|err| error!(error=?err, "failed to get transaction!"))?;

    if project::Entity::find_by_id(node.project_id)
        .one(&txn)
        .await?
        .is_none()
    {
        return Err(WebError::not_found(format!(
            "Project {} not found for new node",
            node.project_id
        )));
    }

    // Clean URL values before saving
    if node.node_type == NodeType::Url {
        node.value = clean_url_value(&node.value);
    }

    let node = node::ActiveModel::from(node);
    let res = node
        .insert(&txn)
        .await
        .inspect_err(|err| error!(error=?err, "Failed to insert node"))?;
    debug!("Saved node: {:?}", res);
    let model = res
        .try_into_model()
        .inspect_err(|err| error!("Failed to convert inserted node to model: {:?}", err))?;
    txn.commit().await.inspect_err(
        |err| error!(error=?err, node=?model, "Failed to commit transaction for new node"),
    )?;
    Ok(Json(model))
}

#[utoipa::path(
    post,
    path = "/api/v1/nodelink",
    request_body = nodelink::Model,
    responses(
        (status = OK, description = "One result ok", body = nodelink::Model)
    )
)]
pub async fn post_nodelink(
    State(state): State<SharedState>,
    Json(nodelink): Json<nodelink::Model>,
) -> Result<Json<nodelink::Model>, WebError> {
    let txn = state.read().await.conn.begin().await?;

    // Validate that the project exists before saving the nodelink
    match nodelink::Entity::find_by_id(nodelink.id).one(&txn).await? {
        Some(_) => {
            // throw an error because it already exists
            Err(WebError {
                status: StatusCode::CONFLICT,
                message: "Nodelink already exists".into(),
            })
        }
        None => {
            // Project doesn't exist
            let nodelink = nodelink.into_active_model();
            let res = nodelink.insert(&txn).await?;
            debug!("Saved nodelink: {:?}", res);
            let model = res.try_into_model()?;
            txn.commit().await?;
            Ok(Json(model))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/project/{project_id}/nodelinks",
    responses(
        (status = OK, description = "One result ok", body = Vec<nodelink::Model>)
    )
)]
pub async fn get_nodelinks_by_project(
    Path(project_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<Vec<nodelink::Model>>, WebError> {
    let nodelinks = nodelink::Entity::find()
        .filter(nodelink::Column::ProjectId.eq(project_id))
        .all(&state.read().await.conn)
        .await?;

    Ok(Json(nodelinks))
}

#[utoipa::path(
    delete,
    path = "/api/v1/node/{id}",
    responses(
        (status = OK, description = "Node deleted successfully", body = String),
        (status = NOT_FOUND, description = "Node not found")
    )
)]
pub async fn delete_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<String>, WebError> {
    let res = node::Entity::delete_by_id(id)
        .exec(&state.read().await.conn)
        .await?;
    match res.rows_affected {
        0 => {
            debug!(node_id = id.to_string(), "Node not found for deletion");
            Err(WebError::not_found(format!("Node {} not found", id)))
        }
        _ => {
            debug!(node_id = id.to_string(), "Deleted node");
            Ok(Json(format!("Node {id} deleted successfully")))
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/node/{id}",
    responses(
        (status = OK, description = "One result ok", body = node::Model)
    )
)]
pub async fn update_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
    Json(mut node): Json<node::Model>,
) -> Result<Json<node::Model>, WebError> {
    let txn = state.read().await.conn.begin().await?;

    // Clean URL values before updating
    if node.node_type == NodeType::Url {
        node.value = clean_url_value(&node.value);
    }

    // Verify node exists first
    match node::Entity::find_by_id(id).one(&txn).await? {
        Some(db_node) => {
            // Update the node ID to match the path parameter
            debug!("Updating node {}: {:?}", id, node);
            let mut db_node = db_node.into_active_model();
            db_node.node_type = Set(node.node_type);
            db_node.display = Set(node.display);
            db_node.value = Set(node.value);
            db_node.updated = Set(Utc::now());
            db_node.notes = Set(node.notes);
            db_node.pos_x = Set(node.pos_x);
            db_node.pos_y = Set(node.pos_y);

            let res = db_node.update(&txn).await?;
            txn.commit().await?;

            Ok(Json(res.try_into_model()?))
        }
        None => {
            debug!("Node {} not found for update", id);
            Err(WebError::not_found(format!("Node {} not found", id)))
        }
    }
}

#[utoipa::path(
    delete,
    path = "/api/v1/nodelink/{id}",
    responses(
        (status = OK, description = "Nodelink deleted successfully", body = ()),
        (status = NOT_FOUND, description = "Nodelink not found")
    )
)]
pub async fn delete_nodelink(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<()>, WebError> {
    let result = nodelink::Entity::delete_by_id(id)
        .exec(&state.read().await.conn)
        .await?;

    match result.rows_affected {
        0 => {
            debug!(
                nodelink_id = id.to_string(),
                "Nodelink not found for deletion"
            );
            Err(WebError::not_found(format!("Nodelink {} not found", id)))
        }
        _ => {
            debug!(nodelink_id = id.to_string(), "Deleted nodelink");
            Ok(Json(()))
        }
    }
}

/// PUT handler to update an existing project
#[utoipa::path(
    put,
    path = "/api/v1/project/{id}",
    request_body = project::Model,
    responses(
        (status = OK, description = "One result ok", body = project::Model)
    )
)]
pub async fn update_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
    Json(project): Json<project::Model>,
) -> Result<Json<project::Model>, WebError> {
    let txn = state.read().await.conn.begin().await?;
    // Verify project exists first
    match project::Entity::find_by_id(id)
        .one(&txn)
        .await
        .inspect_err(|err| error!("Failed to find project {}: {:?}", id, err))?
    {
        Some(db_project) => {
            // Update the project ID to match the path parameter
            debug!("Updating project {}: {:?}", id, project);
            let mut db_project = db_project.into_active_model();
            db_project.description = Set(project.description);
            db_project.name = Set(project.name);
            db_project.tags = Set(project.tags.clone());
            db_project.last_updated = Set(Some(Utc::now()));
            debug!("db_project.is_changed(): {}", db_project.is_changed());
            let res = db_project.update(&txn).await?;
            txn.commit().await?;
            Ok(Json(res.try_into_model()?))
        }
        None => {
            debug!("Project {} not found for update", id);
            Err(WebError::not_found(format!("Project {} not found", id)))
        }
    }
}

/// DELETE handler to delete a project and cascade to nodes/nodelinks
#[utoipa::path(
    delete,
    path = "/api/v1/project/{id}",
    responses(
        (status = OK, description = "Project deleted successfully"),
        (status = NOT_FOUND, description = "Project not found")
    )
)]
pub async fn delete_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<String, WebError> {
    if id == Uuid::nil() {
        debug!("Attempted to delete project with nil UUID");
        return Err(WebError {
            status: StatusCode::BAD_REQUEST,
            message: "Cannot delete project with nil UUID".to_string(),
        });
    }

    let res = project::Entity::delete_by_id(id)
        .exec(&state.read().await.conn)
        .await?;
    if res.rows_affected > 0 {
        info!(
            rows_affected = res.rows_affected,
            id = id.to_string(),
            "Deleted project"
        );
        Ok("Project deleted successfully".to_string())
    } else {
        debug!("Project {} not found for deletion", id);
        Err(WebError::not_found(format!("Project {} not found", id)))
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ProjectExport {
    pub project: project::Model,
    pub nodes: Vec<node::Model>,
    pub nodelinks: Vec<nodelink::Model>,
    pub exported_at: chrono::DateTime<Utc>,
    pub version: String,
    pub attachments: Vec<attachment::Model>,
}

#[derive(Debug, Deserialize)]
pub struct ExportQuery {
    #[serde(default)]
    pub include_attachments: bool,
}

#[utoipa::path(
    get,
    path = "/api/v1/project/{id}/export",
    params(
        ("id" = Uuid, Path, description = "Project ID to export"),
        ("include_attachments" = bool, Query, description = "Whether to include attachments in the export")
    ),
    responses(
        (status = OK, description = "One result ok", body = ProjectExport)
    )
)]
pub async fn export_project(
    Path(id): Path<Uuid>,
    Query(query): Query<ExportQuery>,
    State(state): State<SharedState>,
) -> Result<Json<ProjectExport>, WebError> {
    let txn = state.read().await.conn.begin().await?;

    // Fetch the project
    let project = match project::Entity::find_by_id(id).one(&txn).await? {
        Some(project) => project,
        None => return Err(WebError::not_found(format!("Project {} not found", id))),
    };

    // Fetch nodes
    let nodes = project.find_related(node::Entity).all(&txn).await?;

    // Fetch nodelinks
    let nodelinks = project.find_related(nodelink::Entity).all(&txn).await?;

    // Optionally fetch attachments
    // Get all node IDs for this project
    let node_ids: Vec<Uuid> = nodes.iter().map(|n| n.id).collect();

    // Construct export object
    if query.include_attachments {
        Ok(Json(ProjectExport {
            project,
            nodes,
            nodelinks,
            exported_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            attachments: attachment::Entity::find()
                .filter(attachment::Column::NodeId.is_in(node_ids))
                .all(&txn)
                .await?,
        }))
    } else {
        let attachments: Vec<attachment::Model> = attachment::attachment_list(id)
            .all(&txn)
            .await?
            .into_iter()
            .map(attachment::Model::from)
            .collect();

        Ok(Json(ProjectExport {
            project,
            nodes,
            nodelinks,
            exported_at: Utc::now(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            attachments,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SearchResultType {
    Node(NodeType),
    Project,
    Attachment,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: Uuid,
    pub project_id: Uuid,
    pub title: String,

    pub result_type: SearchResultType,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: String,
}

/// Search across all nodes in all projects
pub async fn search_global(
    State(state): State<SharedState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>, WebError> {
    if query.q.trim().is_empty() {
        return Ok(Json(vec![]));
    }

    let search_term = format!("%{}%", query.q.trim().to_lowercase());
    let txn = state.read().await.conn.begin().await?;

    let mut results: Vec<SearchResult> = Vec::new();

    // Search in node display, value, and notes fields
    let nodes = node::Entity::find()
        .filter(
            node::Column::Display
                .like(&search_term)
                .or(node::Column::Value.like(&search_term))
                .or(node::Column::Notes.like(&search_term)),
        )
        .all(&txn)
        .await?;

    // Add node results
    results.extend(nodes.into_iter().map(|node| SearchResult {
        id: node.id,
        project_id: node.project_id,
        title: node.display,
        result_type: SearchResultType::Node(node.node_type),
    }));

    // Search in attachment filenames
    let attachments = attachment::Entity::find()
        .filter(attachment::Column::Filename.like(&search_term))
        .all(&txn)
        .await?;

    // For each attachment, get the associated node to find project_id
    for attachment_model in attachments {
        if let Some(node_model) = node::Entity::find_by_id(attachment_model.node_id)
            .one(&txn)
            .await?
        {
            results.push(SearchResult {
                id: node_model.id,
                project_id: node_model.project_id,
                title: format!(
                    "{} (attachment: {})",
                    node_model.display, attachment_model.filename
                ),
                result_type: SearchResultType::Node(node_model.node_type),
            });
        }
    }

    // Search in project names, descriptions, and tags
    let projects = project::Entity::find()
        .filter(
            project::Column::Name
                .like(&search_term)
                .or(project::Column::Description.like(&search_term))
                .or(project::Column::Tags.like(&search_term)),
        )
        .all(&txn)
        .await?;

    // For projects, we need to return a representative node or create a special entry
    // Since we need a node_id, we'll find the first node in each matching project
    for project_model in projects {
        if let Some(first_node) = node::Entity::find()
            .filter(node::Column::ProjectId.eq(project_model.id))
            .one(&txn)
            .await?
        {
            results.push(SearchResult {
                id: first_node.id,
                project_id: project_model.id,
                title: format!("Project: {}", project_model.name),
                result_type: SearchResultType::Project,
            });
        }
    }

    Ok(Json(results))
}
