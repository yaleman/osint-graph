use std::net::TcpListener;

use rand::Rng;

pub mod attachment;
pub mod data;
pub mod node;
pub mod nodelink;
pub mod project;
pub mod storage;

pub struct AddrInfo {
    pub addr: String,
    pub port: u16,
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

        let port: u16 = match std::env::var("OSINT_GRAPH_PORT") {
            Ok(val) => val.parse().unwrap_or(8189),
            Err(_) => 8189,
        };

        Self {
            addr: std::env::var("OSINT_GRAPH_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port,
            https,
        }
    }

    pub fn test() -> Self {
        // select a random port
        let mut rng = rand::thread_rng();

        let mut port: u16 = rng.gen_range(32768..65535);
        loop {
            // check if we can connect to it
            println!("checking {}", port);
            if TcpListener::bind(format!("127.0.0.1:{}", port)).is_ok() {
                break;
            }
            port = rng.gen_range(32768..65535);
        }

        Self {
            https: false,
            addr: "127.0.0.69".to_string(),
            port,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addrinfo() {
        let testval = AddrInfo {
            addr: "1.2.3.4".to_string(),
            port: 12345,
            https: true,
        };

        assert_eq!(testval.as_url(), "https://1.2.3.4:12345".to_string());
        assert_eq!(testval.as_addr(), "1.2.3.4:12345".to_string());

        let testval = AddrInfo {
            addr: "1.2.3.4".to_string(),
            port: 12345,
            https: false,
        };
        assert_eq!(testval.as_url(), "http://1.2.3.4:12345".to_string());

        let _ = AddrInfo::from_env();
        let _ = AddrInfo::test();
    }
}
