#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod parser;
mod projects;
mod storage;
pub use app::OsintGraph;

// ----------------------------------------------------------------------------
// When compiling for web:

// #[cfg(target_arch = "wasm32")]
// use eframe::wasm_bindgen::{self, prelude::*};

// This is the entry-point for all the web-assembly.
// This is called once from the HTML.
// It loads the app, installs some callbacks, then returns.
// You can add more callbacks like this if you want to call in to your code.
// #[cfg(target_arch = "wasm32")]
// #[wasm_bindgen]
// pub async fn start(canvas_id: &str) -> Result<(), wasm_bindgen::JsValue> {
//     let app = NodeGraphExample::default();
//     // eframe::
//     // eframe::start_web(canvas_id, Box::new(app))
// }

/// Gets the backend URL
fn get_backend_base_url() -> Result<String, String> {
    use gloo_utils::document;

    let docref = document();
    let url_string = docref.document_uri().map_err(|err| format!("{:?}", err));

    let url = web_sys::Url::new(&url_string?).map_err(|err| format!("{:?}", err))?;

    Ok(format!("{}//{}", url.protocol(), url.host()))
}
