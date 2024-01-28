pub fn get_backend_url() -> String {
    let addr = std::env::var("OSINT_GRAPH_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("OSINT_GRAPH_PORT").unwrap_or_else(|_| "8089".to_string());
    format!("://{}:{}", addr, port)
}
