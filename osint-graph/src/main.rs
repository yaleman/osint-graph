#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use osint_graph::OsintGraph;
    // Redirect `log` message to `console.log` and friends:

    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "osintgraph", // hardcode it
                web_options,
                Box::new(|cc| Box::new(OsintGraph::new(cc))),
            )
            .await
            .expect("failed to start eframe");
    });
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
