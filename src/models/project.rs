use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    PATCH,
    DELETE,
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
            HttpMethod::PUT => write!(f, "PUT"),
            HttpMethod::PATCH => write!(f, "PATCH"),
            HttpMethod::DELETE => write!(f, "DELETE"),
        }
    }
}

impl HttpMethod {
    pub fn all() -> Vec<HttpMethod> {
        vec![
            HttpMethod::GET,
            HttpMethod::POST,
            HttpMethod::PUT,
            HttpMethod::PATCH,
            HttpMethod::DELETE,
        ]
    }

    pub fn next(&self) -> HttpMethod {
        match self {
            HttpMethod::GET => HttpMethod::POST,
            HttpMethod::POST => HttpMethod::PUT,
            HttpMethod::PUT => HttpMethod::PATCH,
            HttpMethod::PATCH => HttpMethod::DELETE,
            HttpMethod::DELETE => HttpMethod::GET,
        }
    }

    pub fn prev(&self) -> HttpMethod {
        match self {
            HttpMethod::GET => HttpMethod::DELETE,
            HttpMethod::POST => HttpMethod::GET,
            HttpMethod::PUT => HttpMethod::POST,
            HttpMethod::PATCH => HttpMethod::PUT,
            HttpMethod::DELETE => HttpMethod::PATCH,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub id: String,
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            method: HttpMethod::GET,
            url: String::new(),
            headers: HashMap::new(),
            body: String::new(),
        }
    }
}

impl Default for Request {
    fn default() -> Self {
        Self::new("New Request".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub requests: Vec<Request>,
}

impl Collection {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            requests: Vec::new(),
        }
    }
}

impl Default for Collection {
    fn default() -> Self {
        Self::new("New Collection".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub collections: Vec<Collection>,
}

impl Project {
    pub fn new(name: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            collections: Vec::new(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self::new("New Project".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Workspace {
    pub projects: Vec<Project>,
}

impl Workspace {
    pub fn new() -> Self {
        Self {
            projects: Vec::new(),
        }
    }

    pub fn with_sample_data() -> Self {
        let mut workspace = Self::new();

        let mut project = Project::new("Sample Project".to_string());
        let mut collection = Collection::new("Sample Collection".to_string());

        let mut request = Request::new("Get Users".to_string());
        request.url = "https://jsonplaceholder.typicode.com/users".to_string();
        request.method = HttpMethod::GET;

        collection.requests.push(request);

        let mut post_request = Request::new("Create User".to_string());
        post_request.url = "https://jsonplaceholder.typicode.com/users".to_string();
        post_request.method = HttpMethod::POST;
        post_request.headers.insert("Content-Type".to_string(), "application/json".to_string());
        post_request.body = r#"{
  "name": "John Doe",
  "email": "john@example.com"
}"#.to_string();

        collection.requests.push(post_request);
        project.collections.push(collection);
        workspace.projects.push(project);

        workspace
    }
}
