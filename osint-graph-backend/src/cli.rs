//! Cli things
//!

use std::path::PathBuf;

use clap::Parser;

pub fn db_path_default() -> String {
    shellexpand::tilde("~/.cache/osint-graph.sqlite3").to_string()
}

#[derive(Parser, Debug)]
pub struct CliOpts {
    #[clap(long, help = "Path to the database file", env = "OSINT_GRAPH_DB_PATH")]
    pub db_path: Option<PathBuf>,

    #[clap(long, help = "Enable debug logging")]
    pub debug: bool,
}
