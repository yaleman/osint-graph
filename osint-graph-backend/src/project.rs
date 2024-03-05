use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use osint_graph_shared::node::Node;
use osint_graph_shared::project::Project;
use uuid::Uuid;

use crate::SharedState;

/// POST handler for project things
pub async fn post_project(
    State(state): State<SharedState>,
    Json(project): Json<Project>,
) -> impl IntoResponse {
    let res = state
        .write()
        .await
        .db
        .save_project(project.clone())
        .unwrap();

    (
        StatusCode::OK,
        serde_json::to_string_pretty(&res).expect("Failed to serialise post project response"),
    )
}

/// Pulls a project from storage.
pub async fn get_project(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> impl IntoResponse {
    let project_id: Uuid = match Uuid::parse_str(&id) {
        Ok(val) => val,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid Uuid".to_string()),
    };

    match state.read().await.db.load_project(&project_id.to_string()) {
        Ok(Some(project)) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&project)
                .expect("Failed to serialise get project response"),
        ),
        Ok(None) => (StatusCode::NOT_FOUND, "Project not found".to_string()),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", err)),
    }
}

pub async fn get_projects(State(state): State<SharedState>) -> impl IntoResponse {
    let res = state.read().await.db.list_projects();
    match res {
        Ok(val) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&val).expect("Failed to serialize project list response"),
        ),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", err)),
    }
}

pub async fn get_node(Path(id): Path<Uuid>, State(state): State<SharedState>) -> impl IntoResponse {
    match state.read().await.db.get_node(id) {
        Ok(val) => match val {
            Some(val) => (
                StatusCode::OK,
                serde_json::to_string(&val).expect("Failed to serialize node"),
            ),
            None => (StatusCode::NOT_FOUND, "".to_string()),
        },
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", err)),
    }
}
pub async fn post_node(
    State(state): State<SharedState>,
    Json(node): Json<Node>,
) -> impl IntoResponse {
    let res = state.write().await.db.save_node(node);
    match res {
        Ok(val) => (
            StatusCode::OK,
            serde_json::to_string_pretty(&val).expect("Failed to serialize project list response"),
        ),
        Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", err)),
    }
}
