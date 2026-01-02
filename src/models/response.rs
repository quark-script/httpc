use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    pub status: u16,
    pub status_text: String,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub time_ms: u128,
    pub size_bytes: usize,
}

impl HttpResponse {
    pub fn new(
        status: u16,
        status_text: String,
        headers: HashMap<String, String>,
        body: String,
        time_ms: u128,
    ) -> Self {
        let size_bytes = body.len();
        Self {
            status,
            status_text,
            headers,
            body,
            time_ms,
            size_bytes,
        }
    }

    pub fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    pub fn is_error(&self) -> bool {
        self.status >= 400
    }

    pub fn formatted_body(&self) -> String {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&self.body) {
            serde_json::to_string_pretty(&json).unwrap_or_else(|_| self.body.clone())
        } else {
            self.body.clone()
        }
    }

    pub fn formatted_size(&self) -> String {
        if self.size_bytes < 1024 {
            format!("{} B", self.size_bytes)
        } else if self.size_bytes < 1024 * 1024 {
            format!("{:.1} KB", self.size_bytes as f64 / 1024.0)
        } else {
            format!("{:.1} MB", self.size_bytes as f64 / (1024.0 * 1024.0))
        }
    }
}

#[derive(Debug, Clone)]
pub struct HttpError {
    pub message: String,
}

impl HttpError {
    pub fn new(message: String) -> Self {
        Self { message }
    }
}

impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
