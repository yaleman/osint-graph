// pub mod db;
pub mod entity;
pub mod migration;

pub mod identifier;
pub mod kvstore;
pub mod middleware;
pub mod project;
pub mod storage;

// mod dev_websocket;
#[cfg(test)]
mod tests;

use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Router,
};
// use dev_websocket::ws_handler;
use project::{
    delete_node, delete_nodelink, delete_project, get_node, get_nodelinks_by_project,
    get_nodes_by_project, get_project, get_projects, post_node, post_nodelink, post_project,
    update_project,
};
use sea_orm::DatabaseConnection;
use std::{borrow::Cow, sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower::{BoxError, ServiceBuilder};
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer, trace::TraceLayer};
use tracing::error;

use crate::project::update_node;

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub conn: DatabaseConnection,
}

impl AppState {
    pub async fn new() -> Self {
        let conn = storage::new().await.unwrap();
        Self { conn }
    }

    #[cfg(test)]
    pub async fn test() -> Self {
        let db = storage::start_db(None)
            .await
            .expect("Failed to start test DB");
        Self { conn: db }
    }
}

pub fn build_app<T>(shared_state: &SharedState) -> Router<T> {
    let static_service = ServeDir::new("./dist/").append_index_html_on_directories(true);

    // Build our application by composing routes
    let router = Router::new()
        .route("/api/v1/node", post(post_node))
        .route("/api/v1/node/:id", get(get_node))
        .route("/api/v1/node/:id", delete(delete_node))
        .route("/api/v1/node/:id", put(update_node))
        .route("/api/v1/nodelink", post(post_nodelink))
        .route("/api/v1/nodelink/:id", delete(delete_nodelink))
        .route(
            "/api/v1/project/:id/nodelinks",
            get(get_nodelinks_by_project),
        )
        .route("/api/v1/project", post(post_project))
        .route("/api/v1/project/:id", get(get_project))
        .route("/api/v1/project/:id", put(update_project))
        .route("/api/v1/project/:id", delete(delete_project))
        .route("/api/v1/project/:id/nodes", get(get_nodes_by_project))
        .route("/api/v1/projects", get(get_projects));

    // #[cfg(debug_assertions)]
    // let router = router.route("/dev-websocket/", get(ws_handler));

    let router = router
        // .route("/api/v1/nodes", get(async { "" }))
        .nest_service("/", static_service.clone())
        .nest_service("/static", static_service);

    router
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(middleware::corslayer())
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    |response: &Response<Body>| {
                        if response.status() == StatusCode::OK {
                            "private, no-transform max-age=0".parse().ok()
                        } else {
                            None
                        }
                    },
                ))
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(shared_state.clone())
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        let msg = "service is overloaded, try again later";
        error!("{}", msg);
        return (StatusCode::SERVICE_UNAVAILABLE, Cow::from(msg));
    }

    let msg = format!("Unhandled internal error: {error}");
    error!("{}", msg);
    (StatusCode::INTERNAL_SERVER_ERROR, Cow::from(msg))
}

#[tokio::test]
async fn test_handle_error() {
    let err = tower::timeout::error::Elapsed::new();
    let res = handle_error(Box::new(err)).await.into_response();
    let expected = (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out")).into_response();

    assert_eq!(res.status(), expected.status());

    let err = tower::load_shed::error::Overloaded::new();
    let res = handle_error(Box::new(err)).await.into_response();
    let expected = (
        StatusCode::SERVICE_UNAVAILABLE,
        Cow::from("service is overloaded, try again later"),
    )
        .into_response();

    assert_eq!(res.status(), expected.status());
}
