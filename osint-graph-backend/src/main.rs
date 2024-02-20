use axum::{
    body::Body,
    error_handling::HandleErrorLayer,
    http::{header, Response, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use osint_graph_backend::{
    project::{get_project, get_projects, post_project},
    SharedState,
};
use osint_graph_shared::AddrInfo;
use tower_http::{services::ServeDir, set_header::SetResponseHeaderLayer};
use tracing::info;

use std::{borrow::Cow, sync::Arc, time::Duration};
use tower::{BoxError, ServiceBuilder};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "osint_graph_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let shared_state = SharedState::default();

    let static_service = ServeDir::new("./dist/").append_index_html_on_directories(true);

    // Build our application by composing routes
    let app = Router::new()
        .route("/api/v1/project", post(post_project))
        .route("/api/v1/project/:id", get(get_project))
        .route("/api/v1/projects", get(get_projects))
        // .route("/api/v1/nodes", get(async { "" }))
        .nest_service("/", static_service.clone())
        .nest_service("/static", static_service)
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(osint_graph_backend::middleware::corslayer())
                .layer(SetResponseHeaderLayer::overriding(
                    header::CACHE_CONTROL,
                    |response: &Response<Body>| {
                        if response.status() == StatusCode::OK {
                            "private no-transform max-age=15".parse().ok()
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
        .with_state(Arc::clone(&shared_state));

    let addrinfo = AddrInfo::from_env();

    // Run our app with hyper
    let listener = tokio::net::TcpListener::bind(&addrinfo.as_addr())
        .await
        .unwrap();
    tracing::info!("listening on {}", addrinfo.as_url());
    axum::serve(listener, app).await.unwrap();
}

async fn handle_error(error: BoxError) -> impl IntoResponse {
    if error.is::<tower::timeout::error::Elapsed>() {
        return (StatusCode::REQUEST_TIMEOUT, Cow::from("request timed out"));
    }

    if error.is::<tower::load_shed::error::Overloaded>() {
        let msg = "service is overloaded, try again later";
        info!("{}", msg);
        return (StatusCode::SERVICE_UNAVAILABLE, Cow::from(msg));
    }

    let msg = format!("Unhandled internal error: {error}");
    info!("{}", msg);
    (StatusCode::INTERNAL_SERVER_ERROR, Cow::from(msg))
}
