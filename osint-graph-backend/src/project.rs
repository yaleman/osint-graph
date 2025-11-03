use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use osint_graph_shared::node::Node;
use osint_graph_shared::project::Project;
use tracing::debug;
use uuid::Uuid;

use crate::db::node::NodeExt;
use crate::db::project::DBProjectExt;
use crate::storage::DBEntity;
use crate::SharedState;

/// POST handler for project things
pub async fn post_project(
    State(state): State<SharedState>,
    Json(project): Json<Project>,
) -> impl IntoResponse {
    project
        .save(&state.read().await.conn)
        .await
        .map_err(|err| {
            debug!("Error saving project: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error saving project: {:?}", err),
            )
        })
        .unwrap();

    (StatusCode::OK, "")
}

/// Pulls a project from storage.
pub async fn get_project(
    Path(id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let res = Project::get(&state.read().await.conn, &id).await;

    match res {
        Ok(Some(project)) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&project)
                .expect("Failed to serialise get project response"), // TODO: handle this better
        ),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found".to_string()),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {:?}", err),
        ),
    }
}

pub async fn get_projects(State(state): State<SharedState>) -> impl IntoResponse {
    match Project::get_all(&state.read().await.conn).await {
        Ok(val) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&val)
                .map_err(|err| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Failed to serialize project list response: {:?}", err),
                    )
                })
                .unwrap(),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {:?}", err),
        ),
    }
}

pub async fn get_node(Path(id): Path<Uuid>, State(state): State<SharedState>) -> impl IntoResponse {
    match Node::get(&state.read().await.conn, &id).await {
        Ok(val) => match val {
            Some(val) => (
                StatusCode::OK,
                serde_json::to_string(&val).expect("Failed to serialize node"),
            ),
            None => (StatusCode::NOT_FOUND, "".to_string()),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {:?}", err),
        ),
    }
}

pub async fn get_nodes_by_project(
    Path(project_id): Path<Uuid>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    match Node::get_by_project_id(&state.read().await.conn, project_id).await {
        Ok(nodes) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&nodes).expect("Failed to serialize nodes list response"),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {:?}", err),
        ),
    }
}

pub async fn post_node(
    State(state): State<SharedState>,
    Json(node): Json<Node>,
) -> impl IntoResponse {
    let conn = &state.read().await.conn;

    // Validate that the project exists before saving the node
    match Project::get(conn, &node.project_id).await {
        Ok(Some(_)) => {
            // Project exists, proceed with saving the node
            let res = node.save(conn).await;
            debug!("Saved node: {:?}", res);
            match res {
                Ok(val) => (
                    StatusCode::OK,
                    serde_json::to_string_pretty(&val).expect("Failed to serialize node response"),
                ),
                Err(err) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error saving node: {:?}", err),
                ),
            }
        }
        Ok(None) => {
            // Project doesn't exist
            debug!("Cannot save node: project {} not found", node.project_id);
            (
                StatusCode::NOT_FOUND,
                format!("Project {} not found", node.project_id),
            )
        }
        Err(err) => {
            // Error checking project
            debug!("Error checking project existence: {:?}", err);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error checking project: {:?}", err),
            )
        }
    }
}
