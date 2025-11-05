use std::{process::ExitCode, sync::Arc};

use clap::Parser;
use osint_graph_backend::{build_app, AppState};
use osint_graph_shared::AddrInfo;

use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = osint_graph_backend::cli::CliOpts::parse();

    let my_filter = match cli.debug {
        true => "osint_graph=debug,tower_http=debug",
        false => "osint_graph=info,tower_http=info",
    };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| my_filter.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let appstate = match AppState::new(&cli).await {
        Ok(state) => state,
        Err(err) => {
            error!("Failed to initialize application state: {:?}", err);
            return ExitCode::FAILURE;
        }
    };
    let shared_state = Arc::new(RwLock::new(appstate));

    let addrinfo = AddrInfo::from_env();

    let app = build_app(&shared_state);

    // Run our app with hyper

    let listener = match tokio::net::TcpListener::bind(&addrinfo.as_addr()).await {
        Ok(val) => {
            info!("Listening on {}", addrinfo.as_url());
            val
        }
        Err(err) => {
            error!("Failed to bind to {}: {:?}", addrinfo.as_url(), err);
            return ExitCode::FAILURE;
        }
    };
    axum::serve(listener, app).await.unwrap();
    ExitCode::SUCCESS
}
