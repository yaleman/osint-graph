//! Axum middleware things
//!

use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};

pub fn corslayer() -> CorsLayer {
    CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods([Method::GET, Method::POST])
        // allow requests from any origin
        .allow_origin(Any)
}
