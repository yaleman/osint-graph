//! Cli things
//!

use std::{net::TcpListener, path::PathBuf};

use clap::Parser;
use osint_graph_shared::Urls;
use rand::Rng;

pub fn db_path_default() -> String {
    shellexpand::tilde("~/.cache/osint-graph.sqlite3").to_string()
}

pub fn test_address() -> String {
    // select a random port
    let mut rng = rand::rng();

    let mut port: u16 = rng.random_range(32768..65535);
    loop {
        // check if we can connect to it
        println!("checking {}", port);
        if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
            break;
        }
        port = rng.random_range(32768..65535);
    }

    format!("127.0.0.69:{}", port)
}

#[derive(Parser, Debug)]
pub struct CliOpts {
    #[clap(long, help = "Path to the database file", env = "OSINT_GRAPH_DB_PATH")]
    pub db_path: Option<PathBuf>,

    #[clap(long, help = "Enable debug logging")]
    pub debug: bool,

    #[clap(
        long,
        env = "OSINT_GRAPH_TLS_CERT",
        help = "Path to TLS certificate file"
    )]
    pub tls_cert: PathBuf,
    #[clap(long, env = "OSINT_GRAPH_TLS_KEY", help = "Path to TLS key file")]
    pub tls_key: PathBuf,

    #[clap(long, env = "OSINT_GRAPH_FRONTEND_URL", help = "URL of the frontend")]
    pub frontend_url: String,
    #[clap(
        long,
        env = "OSINT_GRAPH_LISTENER_ADDRESS",
        help = "Address to listen on",
        default_value = "[::]:9000"
    )]
    pub listener_address: String,
    #[clap(long, env = "OSINT_GRAPH_OIDC_CLIENT_ID", help = "OIDC Client ID")]
    pub oidc_client_id: String,
    #[clap(
        long,
        env = "OSINT_GRAPH_OIDC_DISCOVERY_URL",
        help = "OIDC Discovery URL"
    )]
    pub oidc_discovery_url: String,
}

impl CliOpts {
    pub fn redirect_uri(&self) -> String {
        format!(
            "{}{}",
            self.frontend_url.trim_end_matches('/'),
            Urls::Callback.as_ref()
        )
    }
}
