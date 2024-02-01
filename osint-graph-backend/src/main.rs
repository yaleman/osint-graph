use axum::{
    // body::Bytes,
    error_handling::HandleErrorLayer,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json,
    Router,
};
use tower_http::services::ServeDir;
use tracing::info;

use std::{
    borrow::Cow,
    sync::{Arc, RwLock},
    time::Duration,
};
use tower::{BoxError, ServiceBuilder};
use tower_http::{
    // compression::CompressionLayer, limit::RequestBodyLimitLayer,
    trace::TraceLayer,
    // validate_request::ValidateRequestHeaderLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

async fn get_key(Path(key): Path<String>, State(state): State<SharedState>) -> impl IntoResponse {
    eprintln!("Got get for key: {}", key);
    match state.read().unwrap().db.get(&key) {
        Ok(val) => (StatusCode::OK, val.unwrap_or("".to_string())),
        Err(err) => {
            eprintln!("Failed to get key={} err='{:?}'", key, err);
            (StatusCode::NOT_FOUND, "".to_string())
        }
    }
}

async fn post_set(
    Path(key): Path<String>,
    State(state): State<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    eprintln!("Got post for key: {} value: {:?}", key, payload);

    state
        .write()
        .unwrap()
        .db
        .set(&key, &payload.to_string())
        .expect("Failed to save to DB!");

    "OK"
}

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

type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    db: osint_graph_backend::storage::Storage,
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
