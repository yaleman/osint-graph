use std::sync::Arc;

use osint_graph_backend::{build_app, AppState};
use osint_graph_shared::AddrInfo;

use tokio::sync::RwLock;
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

    let shared_state = Arc::new(RwLock::new(AppState::new().await));

    let addrinfo = AddrInfo::from_env();

    let app = build_app(&shared_state);

    // Run our app with hyper

    let listener = match tokio::net::TcpListener::bind(&addrinfo.as_addr()).await {
        Ok(val) => val,
        Err(err) => {
            tracing::error!("failed to bind to {}: {:?}", addrinfo.as_url(), err);
            return;
        }
    };
    tracing::info!("listening on {}", addrinfo.as_url());
    axum::serve(listener, app).await.unwrap();
}
