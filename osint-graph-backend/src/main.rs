use axum::{
    // body::Bytes,
    error_handling::HandleErrorLayer,
    // extract::{DefaultBodyLimit}
    // extract::{Path, State},
    // handler::Handler,
    http::StatusCode,
    response::IntoResponse,
    // routing::delete,
    // routing::get,
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
        // .route(
        //     "/:key",
        //     // Add compression to `kv_get`
        //     get(kv_get.layer(CompressionLayer::new()))
        //         // But don't compress `kv_set`
        //         .post_service(
        //             kv_set
        //                 .layer((
        //                     DefaultBodyLimit::disable(),
        //                     RequestBodyLimitLayer::new(1024 * 5_000 /* ~5mb */),
        //                 ))
        //                 .with_state(Arc::clone(&shared_state)),
        //         ),
        // )
        // .route("/keys", get(list_keys))
        // Nest our admin routes under `/admin`
        // .nest("/admin", admin_routes())
        // TODO: handle the different serve dirs
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
    tracing::info!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
struct AppState {
    #[allow(dead_code)]
    db: osint_graph_backend::storage::Storage,
}

// async fn kv_get(
//     Path(key): Path<String>,
//     State(state): State<SharedState>,
// ) -> Result<Bytes, StatusCode> {
//     let db = &state.read().unwrap().db;

//     if let Some(value) = db.get(&key) {
//         Ok(value.clone())
//     } else {
//         Err(StatusCode::NOT_FOUND)
//     }
// }

// async fn kv_set(Path(key): Path<String>, State(state): State<SharedState>, bytes: Bytes) {
//     state.write().unwrap().db.insert(key, bytes);
// }

// async fn list_keys(State(state): State<SharedState>) -> String {
//     let db = &state.read().unwrap().db;

//     db.keys()
//         .map(|key| key.to_string())
//         .collect::<Vec<String>>()
//         .join("\n")
// }

// fn admin_routes() -> Router<SharedState> {
//     async fn delete_all_keys(State(state): State<SharedState>) {
//         state.write().unwrap().db.clear();
//     }

//     async fn remove_key(Path(key): Path<String>, State(state): State<SharedState>) {
//         state.write().unwrap().db.remove(&key);
//     }

//     Router::new()
//         .route("/keys", delete(delete_all_keys))
//         .route("/key/:key", delete(remove_key))
//         // Require bearer auth for all admin routes
//         .layer(ValidateRequestHeaderLayer::bearer("secret-token"))
// }

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
