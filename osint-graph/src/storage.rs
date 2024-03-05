use gloo_console::*;

use crate::get_backend_base_url;

#[derive(Clone)]
pub struct Backend {
    #[allow(dead_code)]
    url: String,
}

impl Default for Backend {
    fn default() -> Self {
        let url = get_backend_base_url().expect("Failed to get bsae URL");

        Self { url }
    }
}

impl Backend {
    /// If you don't prepend the endpoint with a forward-slash we're going to do it for you!
    ///
    pub fn make_url(&self, endpoint: &str) -> String {
        if !endpoint.starts_with('/') {
            return format!("{}/{}", self.url, endpoint);
        }
        let res = format!("{}{}", self.url, endpoint);
        info!(
            "Self.url: {} endpoint: {} res: {}",
            &self.url, endpoint, &res
        );
        res
    }
}
