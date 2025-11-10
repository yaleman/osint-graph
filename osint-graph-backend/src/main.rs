use std::{process::ExitCode, sync::Arc};

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use osint_graph_backend::{build_app, cli::CliOpts, AppState};

use tokio::{
    signal::unix::{signal, SignalKind},
    sync::RwLock,
};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

fn export_openapi() {
    use utoipa::OpenApi;
    let openapi = osint_graph_backend::openapi::ApiDoc::openapi();
    let json = serde_json::to_string_pretty(&openapi).expect("Failed to serialize OpenAPI");
    println!("{}", json);
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = osint_graph_backend::cli::CliOpts::parse();

    if cli.export_openapi {
        export_openapi();
        return ExitCode::SUCCESS;
    }

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
    let mut hangup_waiter = match signal(SignalKind::hangup()) {
        Ok(signal) => signal,
        Err(err) => {
            error!("Failed to set up SIGHUP handler: {:?}", err);
            return ExitCode::FAILURE;
        }
    };
    loop {
        tokio::select! {
            res = run_server(&cli, app.clone()) => {
                if let Some(res) = res {
                    return res;
                }
            }
            _ = hangup_waiter.recv() => {
                warn!("Received SIGHUP, shutting down.");
                break
                // TODO: Implement configuration reload logic here

            }
            _ = tokio::signal::ctrl_c() => {
                info!("Received Ctrl-C, shutting down.");
                break
            }
        }
    }
    ExitCode::SUCCESS
}

async fn run_server(cli: &CliOpts, app: Router) -> Option<ExitCode> {
    let tls_server_config = match RustlsConfig::from_pem_file(&cli.tls_cert, &cli.tls_key)
        .await
        .inspect_err(|err| error!(error=?err, "Failed to configure TLS server"))
    {
        Ok(val) => val,
        Err(_) => return Some(ExitCode::FAILURE),
    };
    info!("Starting server on {}", cli.frontend_url);
    axum_server::bind_rustls(
        cli.listener_address.parse().expect("Invalid address"),
        tls_server_config,
    )
    .serve(app.into_make_service())
    .await
    .unwrap();
    None
}
