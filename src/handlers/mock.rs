use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::State,
    http::{Method, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

use crate::{
    config::schema::{MockConfig, MockRoute}, data::CsvStore, persistence::saver::save_request, response::template::{RenderContext, render_response}, validation::body::validate_body,
};

#[derive(Clone)]
pub struct AppState {
    routes: Vec<MockRoute>,
    csv: CsvStore,
    data_dir: String,
}

impl AppState {
    pub fn new(config: MockConfig, csv: CsvStore, data_dir: String) -> Self {
        Self {
            routes: config.routes,
            csv,
            data_dir
        }
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<(MockRoute, HashMap<String, String>)> {
        for route in &self.routes {
            if !method.eq_ignore_ascii_case(&route.method) {
                continue;
            }

            if let Some(params) = match_path(&route.path, path) {
                return Some((route.clone(), params));
            }
        }

        None
    }
}

pub async fn dispatch(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    body: Bytes,
) -> Response {
    let path = uri.path();

    let Some((route, params)) = state.match_route(method.as_str(), path) else {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "route not found" })),
        )
            .into_response();
    };

    let body_json = match parse_body(&body) {
        Ok(value) => value,
        Err(response) => return response,
    };

    if let Some(request_spec) = &route.request {
        if let Some(body_schema) = &request_spec.body {
            let Some(body_value) = &body_json else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "errors": { "body": "missing required body" } })),
                )
                    .into_response();
            };

            if let Err(errors) = validate_body(body_schema, body_value) {
                return (StatusCode::BAD_REQUEST, Json(json!({ "errors": errors }))).into_response();
            }
        }
    }

    let body_for_context = body_json.unwrap_or(json!(null));
    let ctx = RenderContext {
        params: &params,
        body: &body_for_context,
        index: None,
        csv: &state.csv,
    };

    let rendered = render_response(&route.response.body, &ctx);
    let status = StatusCode::from_u16(route.response.status).unwrap_or(StatusCode::OK);

    // save response 

    if let Some(save_spec) = &route.save {
        if let Err(err) = save_request(save_spec, &rendered, &params, &state.data_dir) {
            return (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({
                "error": format!("failed to save: {:?}", err)
            }))).into_response();
        }
    }

    (status, Json(rendered)).into_response()
}

fn parse_body(body: &Bytes) -> Result<Option<serde_json::Value>, Response> {
    if body.is_empty() {
        return Ok(None);
    }

    match serde_json::from_slice(body) {
        Ok(value) => Ok(Some(value)),
        Err(err) => Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": format!("invalid json: {err}") })),
        )
            .into_response()),
    }
}

fn match_path(pattern: &str, path: &str) -> Option<HashMap<String, String>> {
    let pattern_parts: Vec<&str> = pattern.split('/').filter(|part| !part.is_empty()).collect();
    let path_parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();

    if pattern_parts.len() != path_parts.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_part, path_part) in pattern_parts.iter().zip(path_parts.iter()) {
        if let Some(name) = pattern_part.strip_prefix(':') {
            params.insert(name.to_string(), (*path_part).to_string());
        } else if pattern_part != path_part {
            return None;
        }
    }

    Some(params)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_path_params() {
        let params = match_path("/api/users/:id", "/api/users/42").unwrap();
        assert_eq!(params.get("id").map(String::as_str), Some("42"));
    }

    #[test]
    fn rejects_wrong_segment_count() {
        assert!(match_path("/api/users/:id", "/api/users").is_none());
    }
}
