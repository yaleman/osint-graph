use std::sync::Arc;

use tokio::sync::RwLock;

pub mod identifier;
pub mod kvstore;
pub mod project;
pub mod storage;

pub type SharedState = Arc<RwLock<AppState>>;

#[derive(Default)]
pub struct AppState {
    pub db: storage::Storage,
}
