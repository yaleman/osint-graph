use axum::extract::{Path, State};
use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderValue, StatusCode};
use axum::response::IntoResponse;
use axum::Json;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, IntoActiveModel, ModelTrait, QueryFilter,
    TryIntoModel,
};
use serde::{Deserialize, Serialize};
use sqlx::types::chrono::Utc;
use tracing::{debug, error};
use uuid::Uuid;

use crate::entity::{node, nodelink, project};
use crate::SharedState;

/// POST handler for project things
pub async fn post_project(
    State(state): State<SharedState>,
    Json(project): Json<project::Model>,
) -> Result<Json<project::Model>, WebError> {
    let project = match project::Entity::find()
        .filter(project::Column::Id.eq(project.id))
        .one(&state.read().await.conn)
        .await?
    {
        Some(val) => {
            let mut target_project = val.into_active_model();
            target_project
                .description
                .set_if_not_equals(project.description);
            target_project.name.set_if_not_equals(project.name);
            target_project.tags.set_if_not_equals(project.tags.clone());
            target_project
                .last_updated
                .set_if_not_equals(Some(Utc::now()));

            target_project
                .save(&state.read().await.conn)
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
    pub fn not_found(message: String) -> Self {
        WebError {
            status: StatusCode::NOT_FOUND,
            message,
        }
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
pub async fn get_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, WebError> {
    let res = project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .one(&state.read().await.conn)
        .await?;

    match res {
        Some(project) => Ok((
            StatusCode::OK,
            serde_json::to_string_pretty(&project)
                .expect("Failed to serialise get project response"), // TODO: handle this better
        )),
        None => Ok((StatusCode::NOT_FOUND, "Project not found".to_string())),
    }
}

pub async fn get_projects(State(state): State<SharedState>) -> Result<impl IntoResponse, WebError> {
    let val = project::Entity::find()
        .all(&state.read().await.conn)
        .await?;
    Ok((StatusCode::OK, serde_json::to_string_pretty(&val)?))
}

pub async fn get_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, WebError> {
    match node::Entity::find()
        .filter(node::Column::Id.eq(id))
        .one(&state.read().await.conn)
        .await?
    {
        Some(val) => Ok((
            StatusCode::OK,
            serde_json::to_string(&val).expect("Failed to serialize node"),
        )),
        None => Ok((StatusCode::NOT_FOUND, "".to_string())),
    }
}

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

pub async fn post_node(
    State(state): State<SharedState>,
    Json(node): Json<node::Model>,
) -> Result<Json<node::Model>, WebError> {
    let conn = &state.read().await.conn;

    // Validate that the project exists before saving the node
    match project::Entity::find()
        .filter(project::Column::Id.eq(node.project_id))
        .one(conn)
        .await
        .inspect_err(|err| error!("Failed to find project {}: {:?}", node.project_id, err))?
    {
        Some(_) => {
            // Project exists, proceed with saving the node
            let node = node::ActiveModel::from(node);
            let res = node
                .insert(conn)
                .await
                .inspect_err(|err| error!("Failed to insert node: {:?}", err))?;
            debug!("Saved node: {:?}", res);
            let model = res
                .try_into_model()
                .inspect_err(|err| error!("Failed to convert inserted node to model: {:?}", err))?;

            Ok(Json(model))
        }
        None => {
            // Project doesn't exist
            debug!("Cannot save node: project {} not found", node.project_id);
            Err(WebError::not_found(format!(
                "Project {} not found",
                node.project_id
            )))
        }
    }
}

pub async fn post_nodelink(
    State(state): State<SharedState>,
    Json(nodelink): Json<nodelink::Model>,
) -> Result<Json<nodelink::Model>, WebError> {
    let conn = &state.read().await.conn;

    // Validate that the project exists before saving the nodelink
    match nodelink::Entity::find()
        .filter(nodelink::Column::Id.eq(nodelink.id))
        .one(conn)
        .await?
    {
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
            let res = nodelink.insert(conn).await?;
            debug!("Saved nodelink: {:?}", res);
            let model = res.try_into_model()?;
            Ok(Json(model))
        }
    }
}

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

pub async fn delete_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<impl IntoResponse, WebError> {
    let conn = &state.read().await.conn;
    // find the node
    let node = match node::Entity::find_by_id(id).one(conn).await? {
        Some(node) => node,
        None => {
            debug!("Node {} not found for deletion", id);
            return Ok((StatusCode::NOT_FOUND, format!("Node {} not found", id)));
        }
    };

    node.delete(conn).await?;

    Ok((StatusCode::OK, "Node deleted successfully".to_string()))
}

pub async fn update_node(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
    Json(node): Json<node::Model>,
) -> Result<Json<node::Model>, WebError> {
    let conn = &state.read().await.conn;
    // Verify node exists first
    match node::Entity::find()
        .filter(node::Column::Id.eq(id))
        .one(conn)
        .await?
    {
        Some(db_node) => {
            // Update the node ID to match the path parameter
            debug!("Updating node {}: {:?}", id, node);
            let mut db_node = db_node.into_active_model();
            db_node.node_type.set_if_not_equals(node.node_type);
            db_node.display.set_if_not_equals(node.display);
            db_node.value.set_if_not_equals(node.value);
            db_node.updated.set_if_not_equals(Utc::now());
            db_node.notes.set_if_not_equals(node.notes);
            db_node.pos_x.set_if_not_equals(node.pos_x);
            db_node.pos_y.set_if_not_equals(node.pos_y);
            db_node
                .attachments
                .set_if_not_equals(node.attachments.clone());

            let res = db_node.save(conn).await?;
            Ok(Json(res.try_into_model()?))
        }
        None => {
            debug!("Node {} not found for update", id);
            Err(WebError::not_found(format!("Node {} not found", id)))
        }
    }
}

pub async fn delete_nodelink(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<()>, WebError> {
    let conn = &state.read().await.conn;
    let nodelink = nodelink::Entity::find_by_id(id).one(conn).await?;

    match nodelink {
        Some(nodelink) => {
            debug!("Deleted nodelink: {}", id);
            nodelink.delete(conn).await?;
            Ok(Json(()))
        }
        None => {
            debug!("Nodelink {} not found for deletion", id);
            Err(WebError::not_found(format!("Nodelink {} not found", id)))
        }
    }
}

/// PUT handler to update an existing project
pub async fn update_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
    Json(project): Json<project::Model>,
) -> Result<Json<project::Model>, WebError> {
    let conn = &state.read().await.conn;
    // Verify project exists first
    match project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .one(conn)
        .await
        .inspect_err(|err| error!("Failed to find project {}: {:?}", id, err))?
    {
        Some(db_project) => {
            // Update the project ID to match the path parameter
            debug!("Updating project {}: {:?}", id, project);
            let mut db_project = db_project.into_active_model();
            db_project
                .description
                .set_if_not_equals(project.description);
            db_project.name.set_if_not_equals(project.name);
            db_project.tags.set_if_not_equals(project.tags.clone());
            db_project.last_updated.set_if_not_equals(Some(Utc::now()));

            let res = db_project.save(conn).await?;
            Ok(Json(res.try_into_model()?))
        }
        None => {
            debug!("Project {} not found for update", id);
            Err(WebError::not_found(format!("Project {} not found", id)))
        }
    }
}

/// DELETE handler to delete a project and cascade to nodes/nodelinks
pub async fn delete_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let conn = &state.read().await.conn;

    // Verify project exists first
    match project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .one(conn)
        .await
    {
        Ok(Some(project)) => {
            // Delete the project - cascade should handle nodes and nodelinks automatically
            match project.delete(conn).await {
                Ok(res) => {
                    debug!(
                        res = format!("{:?}", res),
                        id = id.to_string(),
                        "Deleted project"
                    );
                    (StatusCode::OK, "Project deleted successfully".to_string())
                }
                Err(err) => {
                    debug!("Error deleting project {}: {:?}", id, err);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Error deleting project: {:?}", err),
                    )
                }
            }
        }
        Ok(None) => {
            debug!("Project {} not found for deletion", id);
            (StatusCode::NOT_FOUND, format!("Project {} not found", id))
        }
        Err(err) => {
            debug!("Error checking project existence: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error checking project: {:?}", err),
            )
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectExport {
    pub project: project::Model,
    pub nodes: Vec<node::Model>,
    pub nodelinks: Vec<nodelink::Model>,
}

pub async fn export_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> Result<Json<ProjectExport>, WebError> {
    let conn = &state.read().await.conn;

    // Fetch the project
    let project = match project::Entity::find()
        .filter(project::Column::Id.eq(id))
        .one(conn)
        .await?
    {
        Some(project) => project,
        None => return Err(WebError::not_found(format!("Project {} not found", id))),
    };

    // Fetch nodes
    let nodes = node::Entity::find()
        .filter(node::Column::ProjectId.eq(id))
        .all(conn)
        .await?;

    // Fetch nodelinks
    let nodelinks = nodelink::Entity::find()
        .filter(nodelink::Column::ProjectId.eq(id))
        .all(conn)
        .await?;

    // Construct export object
    Ok(Json(ProjectExport {
        project,
        nodes,
        nodelinks,
    }))
}
