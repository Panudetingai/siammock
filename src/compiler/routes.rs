use std::collections::HashSet;

use crate::config::schema::MockConfig;

use super::diagnostics::Diagnostic;
use super::span::find_text;

const VALID_METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

pub fn validate_routes(config: &MockConfig, source: &str, out: &mut Vec<Diagnostic>) {
    let mut seen = HashSet::new();

    for (i, route) in config.routes.iter().enumerate() {
        let base = format!("routes[{i}]");

        if !VALID_METHODS
            .iter()
            .any(|method| method.eq_ignore_ascii_case(&route.method))
        {
            let needle = format!("\"method\": \"{}\"", route.method);
            let (line, column) = find_text(source, &needle).unwrap_or((1, 1));
            out.push(Diagnostic::error(
                "INVALID_HTTP_METHOD",
                format!("{base}.method"),
                line,
                column,
                format!("HTTP method '{}' is not supported", route.method),
                Some(format!("Use one of: {}", VALID_METHODS.join(", "))),
            ));
        }

        if !route.path.starts_with('/') {
            let needle = format!("\"path\": \"{}\"", route.path);
            let (line, column) = find_text(source, &needle).unwrap_or((1, 1));
            out.push(Diagnostic::error(
                "INVALID_PATH",
                format!("{base}.path"),
                line,
                column,
                format!("path '{}' must start with '/'", route.path),
                Some("Example: \"/api/v1/users\"".into()),
            ));
        }

        if route.response.status < 100 || route.response.status > 599 {
            let needle = format!("\"status\": {}", route.response.status);
            let (line, column) = find_text(source, &needle).unwrap_or((1, 1));
            out.push(Diagnostic::error(
                "INVALID_STATUS",
                format!("{base}.response.status"),
                line,
                column,
                format!(
                    "response status {} is out of range (100-599)",
                    route.response.status
                ),
                None,
            ));
        }

        let key = format!("{}:{}", route.method.to_ascii_uppercase(), route.path);
        if !seen.insert(key.clone()) {
            out.push(Diagnostic::error(
                "DUPLICATE_ROUTE",
                base.clone(),
                1,
                1,
                format!("duplicate route: {key}"),
                Some("Each path + method pair should appear only once".into()),
            ));
        }

        if let Some(request) = &route.request {
            if let Some(body) = &request.body {
                validate_request_body(body, &format!("{base}.request.body"), source, out);
            }
        }
    }
}

fn validate_request_body(
    body: &std::collections::HashMap<String, serde_json::Value>,
    path: &str,
    source: &str,
    out: &mut Vec<Diagnostic>,
) {
    for (field, value) in body {
        let Some(text) = value.as_str() else {
            continue;
        };

        if is_validatable_type(text) || is_example_value(text) {
            if is_validatable_type(text) && !is_known_type_descriptor(text) {
                let field_path = format!("{path}.{field}");
                let needle = format!("\"{field}\": \"{text}\"");
                let (line, column) = find_text(source, &needle).unwrap_or((1, 1));
                out.push(Diagnostic::error(
                    "INVALID_BODY_TYPE",
                    field_path,
                    line,
                    column,
                    format!("unknown body type descriptor '{text}'"),
                    Some(
                        "Use: string, number, boolean, array, object, string[], number[], boolean[], or add (required)".into(),
                    ),
                ));
            }
            continue;
        }

        if text.contains("{{") {
            let field_path = format!("{path}.{field}");
            let needle = format!("\"{field}\": \"{text}\"");
            let (line, column) = find_text(source, &needle).unwrap_or((1, 1));
            out.push(Diagnostic::warning(
                "AMBIGUOUS_BODY_VALUE",
                field_path,
                line,
                column,
                format!("field '{field}' looks like an example value, not a type descriptor"),
                Some("Use type descriptors like \"string (required)\" to validate incoming requests".into()),
            ));
        }
    }
}

fn is_validatable_type(value: &str) -> bool {
    is_known_type_descriptor(value) || value.contains("(required")
}

fn is_known_type_descriptor(value: &str) -> bool {
    matches!(
        normalize_type(value),
        Some(
            "string" | "number" | "boolean" | "array" | "object" | "string[]" | "number[]"
                | "boolean[]"
        )
    )
}

fn normalize_type(value: &str) -> Option<&str> {
    let base = value.split_whitespace().next()?.trim();
    match base {
        "string" | "number" | "boolean" | "array" | "object" | "string[]" | "number[]"
        | "boolean[]" => Some(base),
        _ => None,
    }
}

fn is_example_value(value: &str) -> bool {
    value.contains('@')
        || value.starts_with("http")
        || value.chars().any(|ch| ch.is_ascii_digit())
        || value.contains(' ')
}
