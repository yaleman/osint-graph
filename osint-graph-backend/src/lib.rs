pub mod attachment;
pub mod auth;
pub mod cli;
pub mod entity;
pub mod identifier;
pub mod logging;
pub mod middleware;
pub mod migration;
pub mod oauth;
pub mod openapi;
pub mod project;
pub mod storage;
#[cfg(test)]
mod tests;
pub mod tls;

use attachment::{
    delete_attachment, download_attachment, list_attachments, upload_attachment, view_attachment,
};
use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    http::{header, Response, StatusCode},
    middleware::from_fn_with_state,
    routing::{delete, get, post},
    Router,
};
use osint_graph_shared::{error::OsintError, Urls};
use project::{
    delete_node, delete_nodelink, delete_project, get_node, get_nodelinks_by_project,
    get_nodes_by_project, get_project, get_projects, post_node, post_nodelink, post_project,
    update_project,
};
use sea_orm::DatabaseConnection;
use sqlx::{Pool, Sqlite};
use std::{sync::Arc, time::Duration};
use tokio::sync::RwLock;
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    compression::CompressionLayer, services::ServeDir, set_header::SetResponseHeaderLayer,
};
use tower_sessions::{cookie::time, Expiry, SessionManagerLayer};
use tracing::error;

use crate::{
    attachment::update_attachment,
    cli::{db_path_default, CliOpts},
    logging::logging_layer,
    oauth::{middleware::require_auth, OAuthClient},
    project::{export_project, update_node, WebError},
};

pub type SharedState = Arc<RwLock<AppState>>;

pub struct AppState {
    pub conn: DatabaseConnection,

    pub oauth_client: Option<Arc<OAuthClient>>,
}

impl AppState {
    pub async fn new(cli: &CliOpts) -> Result<Self, OsintError> {
        let conn = storage::new(&cli.db_path.clone().unwrap_or(db_path_default().into())).await?;
        Ok(Self {
            oauth_client: Some(Arc::new(
                OAuthClient::new(
                    &cli.oidc_discovery_url,
                    &cli.oidc_client_id,
                    &cli.redirect_uri(),
                    Arc::new(conn.clone()),
                )
                .await?,
            )),
            conn,
        })
    }

    #[cfg(test)]
    pub async fn test() -> Self {
        let db = storage::start_db(None)
            .await
            .expect("Failed to start test DB");
        Self {
            conn: db,
            oauth_client: None,
        }
    }
}

pub async fn build_app(
    shared_state: &SharedState,
    db_pool: Pool<Sqlite>,
    enable_oauth: bool,
) -> Router {
    // Create session layer (secure cookies for HTTPS)
    let session_store = tower_sessions_sqlx_store::SqliteStore::new(db_pool);

    // Migrate the session store to create the sessions table
    session_store
        .migrate()
        .await
        .expect("Failed to migrate session store");

    let session_layer = SessionManagerLayer::new(session_store)
        .with_secure(true) // HTTPS only - secure cookies
        .with_expiry(Expiry::OnInactivity(time::Duration::hours(1)));

    let static_service = ServeDir::new("./dist/").append_index_html_on_directories(true);

    // Build our application by composing routes
    let protected_routes = Router::new()
        .route("/api/v1/node", post(post_node))
        .route(
            "/api/v1/node/{id}",
            get(get_node).delete(delete_node).put(update_node),
        )
        .route(
            "/api/v1/node/{id}/attachment",
            post(upload_attachment).layer(DefaultBodyLimit::max(100 * 1024 * 1024)), // 100MB limit
        )
        .route("/api/v1/node/{id}/attachments", get(list_attachments))
        .route(
            "/api/v1/attachment/{attachment_id}",
            get(download_attachment)
                .delete(delete_attachment)
                .patch(update_attachment),
        )
        .route(
            "/api/v1/attachment/{attachment_id}/view",
            get(view_attachment),
        )
        .route("/api/v1/nodelink", post(post_nodelink))
        .route("/api/v1/nodelink/{id}", delete(delete_nodelink))
        .route(
            "/api/v1/project/{id}/nodelinks",
            get(get_nodelinks_by_project),
        )
        .route("/api/v1/project", post(post_project))
        .route(
            "/api/v1/project/{id}",
            get(get_project).put(update_project).delete(delete_project),
        )
        .route("/api/v1/project/{id}/nodes", get(get_nodes_by_project))
        .route("/api/v1/projects", get(get_projects))
        .route("/api/v1/project/{id}/export", get(export_project))
        .nest_service("/static", static_service.clone())
        .merge(openapi::api_route())
        .fallback_service(static_service);

    let res = if enable_oauth {
        // Auth routes should NOT have the require_auth middleware
        Router::new()
            .route(Urls::Login.as_ref(), get(auth::auth_login))
            .route(Urls::Callback.as_ref(), get(auth::auth_callback))
            .route(Urls::Logout.as_ref(), get(auth::auth_logout))
            .merge(protected_routes.layer(from_fn_with_state(shared_state.clone(), require_auth)))
    } else {
        protected_routes
    };

    res
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                .layer(session_layer)
                .layer(
                    CompressionLayer::new()
                        .gzip(true)
                        .deflate(true)
                        .quality(tower_http::CompressionLevel::Best),
                )
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
                .layer(logging_layer()),
        )
        .with_state(shared_state.clone())
}

async fn handle_error(error: BoxError) -> WebError {
    if error.is::<tower::timeout::error::Elapsed>() {
        return WebError::new(StatusCode::REQUEST_TIMEOUT, "request timed out");
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        let msg = "service is overloaded, try again later";
        error!("{}", msg);
        return WebError::new(StatusCode::SERVICE_UNAVAILABLE, msg);
    }

    let msg = format!("Unhandled internal error: {error}");
    error!("{}", msg);
    WebError::internal_server_error(msg)
}
