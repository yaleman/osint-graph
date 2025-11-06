use std::{process::ExitCode, sync::Arc};

use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use osint_graph_backend::{build_app, AppState};

use tokio::sync::RwLock;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> ExitCode {
    let cli = osint_graph_backend::cli::CliOpts::parse();

    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

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
    let db_pool = appstate.conn.get_sqlite_connection_pool().clone();

    let shared_state = Arc::new(RwLock::new(appstate));

    let app = build_app(&shared_state, db_pool, true).await;

    // Run our app with hyper

    let tls_server_config = match RustlsConfig::from_pem_file(&cli.tls_cert, &cli.tls_key)
        .await
        .inspect_err(|err| error!(error=?err, "Failed to configure TLS server"))
    {
        Ok(val) => val,
        Err(_) => return ExitCode::FAILURE,
    };
    info!("Starting server on {}", cli.frontend_url);
    axum_server::bind_rustls(
        cli.listener_address.parse().expect("Invalid address"),
        tls_server_config,
    )
    .serve(app.into_make_service())
    .await
    .unwrap();
    ExitCode::SUCCESS
}
