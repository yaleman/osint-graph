use axum::extract::{Path, State};
// use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;

use crate::SharedState;

pub async fn get_key(
    Path(key): Path<String>,
    State(_state): State<SharedState>,
) -> impl IntoResponse {
    eprintln!("Got get for key: {}", key);
    unimplemented!();
    // match state.read().await.conn.get(&key) {
    //     Ok(val) => (StatusCode::OK, val.unwrap_or("".to_string())),
    //     Err(err) => {
    //         eprintln!("Failed to get key={} err='{:?}'", key, err);
    //         (StatusCode::NOT_FOUND, "".to_string())
    //     }
    // }
}

pub async fn post_set(
    Path(key): Path<String>,
    State(_state): State<SharedState>,
    Json(payload): Json<serde_json::Value>,
) -> &'static str {
    eprintln!("Got post for key: {} value: {:?}", key, payload);

    unimplemented!("Got post for key: {} value: {:?}", key, payload);
    // state
    //     .write()
    //     .await
    //     .conn
    //     .set(&key, &payload.to_string())
    //     .expect("Failed to save to DB!");

    // "OK"
}
