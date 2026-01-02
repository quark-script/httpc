use std::collections::HashMap;
use std::time::Instant;

use anyhow::{Context, Result};
use reqwest::Client;

use crate::models::{HttpMethod, HttpResponse, Request};

pub struct HttpClient {
    pub client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn execute(&self, request: &Request) -> Result<HttpResponse> {
        let start = Instant::now();

        let mut builder = match request.method {
            HttpMethod::GET => self.client.get(&request.url),
            HttpMethod::POST => self.client.post(&request.url),
            HttpMethod::PUT => self.client.put(&request.url),
            HttpMethod::PATCH => self.client.patch(&request.url),
            HttpMethod::DELETE => self.client.delete(&request.url),
        };

        for (key, value) in &request.headers {
            builder = builder.header(key, value);
        }

        if !request.body.is_empty() && matches!(request.method, HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH) {
            builder = builder.body(request.body.clone());
        }

        let response = builder.send().await.context("Failed to send request")?;

        let elapsed = start.elapsed().as_millis();
        let status = response.status().as_u16();
        let status_text = response.status().canonical_reason()
            .unwrap_or("Unknown")
            .to_string();

        let headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_string(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();

        let body = response.text().await.context("Failed to read response body")?;

        Ok(HttpResponse::new(status, status_text, headers, body, elapsed))
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}
