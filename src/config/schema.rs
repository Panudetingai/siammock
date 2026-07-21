use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
pub struct MockConfig {
    pub routes: Vec<MockRoute>,
}

#[derive(Deserialize, Clone)]
pub struct MockRoute {
    pub path: String,
    pub method: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub summary: Option<String>,
    pub request: Option<RequestSpec>,
    pub response: ResponseSpec,
}

#[derive(Deserialize, Clone)]
pub struct RequestSpec {
    #[serde(default)]
    #[allow(dead_code)]
    pub headers: Option<HashMap<String, Value>>,
    #[serde(default)]
    #[allow(dead_code)]
    pub query_params: Option<HashMap<String, Value>>,
    #[serde(default)]
    pub body: Option<HashMap<String, Value>>,
}

#[derive(Deserialize, Clone)]
pub struct ResponseSpec {
    pub status: u16,
    #[serde(default)]
    #[allow(dead_code)]
    pub headers: Option<HashMap<String, Value>>,
    pub body: Value,
}
