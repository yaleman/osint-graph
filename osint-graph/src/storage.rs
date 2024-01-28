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
    /// Ensure you prepend a `/` to the endpoint
    ///
    #[allow(dead_code)]
    pub fn make_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.url, endpoint)
    }
}

use gloo_net::http::Request;

use crate::get_backend_base_url;

impl Backend {
    #[allow(dead_code)]
    pub async fn get(&self, key: &str) -> Option<String> {
        let resp = Request::get(&self.make_url(&format!("/get/{}", key)))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        match resp.text().await {
            Ok(text) => Some(text),
            Err(e) => {
                gloo_console::error!("Error getting string: {}", e.to_string());
                None
            }
        }
    }

    #[allow(dead_code)]
    pub async fn set(&mut self, key: &str, value: String) {
        let resp = Request::post(&self.make_url(&format!("/set/{}", key)))
            .json(&value)
            .unwrap()
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    // fn flush(&mut self) {}
}
