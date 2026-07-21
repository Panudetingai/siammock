use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct MockConfig {
    routes: Vec<MockRoute>,
}

#[derive(Deserialize, Clone)]
struct MockRoute {
    path: String,
    method: String,
    request: Option<RequestSpec>,
    response: ResponseSpec,
}

#[derive(Deserialize, Clone)]
struct RequestSpec {
    body: HashMap<String, String>, // "name": "string"
}

#[derive(Deserialize, Clone)]
struct ResponseSpec {
    status: u16,
    body: serde_json::Value,
}