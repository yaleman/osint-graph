pub mod data;
pub mod node;
pub mod project;
pub mod storage;

pub struct AddrInfo {
    pub addr: String,
    pub port: String,
    pub https: bool,
}

impl AddrInfo {
    pub fn as_url(&self) -> String {
        let scheme = match self.https {
            true => "https",
            false => "http",
        };
        format!("{}://{}:{}", scheme, self.addr, self.port)
    }

    pub fn as_addr(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }

    pub fn from_env() -> Self {
        let https = match std::env::var("OSINT_GRAPH_HTTPS") {
            Ok(val) => val == "true",
            Err(_) => false,
        };

        Self {
            addr: std::env::var("OSINT_GRAPH_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("OSINT_GRAPH_PORT").unwrap_or_else(|_| "8189".to_string()),
            https,
        }
    }
}
