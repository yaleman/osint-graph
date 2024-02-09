use ehttp::{fetch, Headers, Request, Response};
use gloo_console::{error, info};

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

use crate::get_backend_base_url;

impl Backend {
    #[allow(dead_code)]
    pub fn get(&self, key: &str, egui_ctx: &eframe::egui::Context) {
        let req = Request::get(self.make_url(&format!("/get/{}", key)));
        let egui_ctx = egui_ctx.clone();
        let key = key.to_string();
        fetch(req, move |response: Result<Response, String>| {
            match response {
                Ok(resp) => {
                    if resp.status != 200 {
                        error!("Failed to perform get request: {}", resp.status);
                    } else {
                        info!(format!(
                            "Got response for get: {} status: {} value: {:?}",
                            key,
                            resp.status,
                            resp.text()
                        ));
                    }
                }
                Err(err) => {
                    println!("Failed to perform get request: {}", err);
                }
            }

            egui_ctx.request_repaint();
        });
        // .expect("Failed to perform get request");
        // assert_eq!(resp.status, 200);
        // resp.text().map(|s| s.to_string())
    }

    pub fn set(&mut self, key: &str, value: &str, egui_ctx: &eframe::egui::Context) {
        let url = self.make_url(&format!("/set/{}", key));
        info!("Dest url: {}", &url);
        let headers: Headers = Headers::new(&[("Content-Type", "application/json; charset=utf-8")]);
        let req = Request {
            url,
            method: "POST".into(),
            headers,
            body: serde_json::to_string(value)
                .expect("Failed to serialise data!")
                .into(),
        };
        // ::post(&url, value.as_bytes().into());
        // req.set_header("Content-Type", "text/plain");

        let egui_ctx = egui_ctx.clone();
        fetch(req, move |response: Result<Response, String>| {
            match response {
                Ok(resp) => {
                    if resp.status != 200 {
                        error!(format!(
                            "Failed to perform set request: status={} body={}",
                            resp.status,
                            resp.text().unwrap_or("No body")
                        ));
                    } else {
                        info!("Got response: {}", resp.status);
                    }
                }
                Err(err) => {
                    println!("Failed to perform get request: {}", err);
                }
            }

            egui_ctx.request_repaint();
        });
    }
}
