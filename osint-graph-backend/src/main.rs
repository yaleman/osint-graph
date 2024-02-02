use axum::{
    error_handling::HandleErrorLayer,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use osint_graph_backend::{
    kvstore::{get_key, post_set},
    SharedState,
};
use tower_http::services::ServeDir;
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

    // Build our application by composing routes
    let app = Router::new()
        .route("/set/:key", post(post_set))
        .route("/get/:key", get(get_key))
        .nest_service(
            "/",
            ServeDir::new("./dist/").append_index_html_on_directories(true),
        )
        .nest_service(
            "/static",
            ServeDir::new("./dist").append_index_html_on_directories(true),
        )
        // Add middleware to all routes
        .layer(
            ServiceBuilder::new()
                // Handle errors from middleware
                .layer(HandleErrorLayer::new(handle_error))
                .load_shed()
                .concurrency_limit(1024)
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http()),
        )
        .with_state(Arc::clone(&shared_state));

    let addr = std::env::var("OSINT_GRAPH_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("OSINT_GRAPH_PORT").unwrap_or_else(|_| "8089".to_string());
    let fulladdr = format!("{}:{}", addr, port);
    // Run our app with hyper
    let listener = tokio::net::TcpListener::bind(&fulladdr).await.unwrap();
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
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
