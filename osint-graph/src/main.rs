#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds

// When compiling to web
fn main() {
    #[cfg(target_arch = "wasm32")]
    use osint_graph::OsintGraph;

    // Redirect `log` message to `console.log` and friends:
    #[cfg(target_arch = "wasm32")]
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    #[cfg(target_arch = "wasm32")]
    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        #[cfg(target_arch = "wasm32")]
        eframe::WebRunner::new()
            .start(
                "osintgraph", // hardcode it
                web_options,
                Box::new(|cc| Box::new(OsintGraph::new(cc))),
            )
            .await
            .expect("Failed to start eframe webui, can't run page!");
    });
}
