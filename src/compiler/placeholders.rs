use serde_json::Value;

use crate::config::schema::MockConfig;

use super::diagnostics::Diagnostic;
use super::span::find_text;

const STATIC_PLACEHOLDERS: &[&str] = &[
    "uuid",
    "timestamp",
    "random_number",
    "jwt_token",
    "random_string",
    "thai_name",
    "en_name",
    "email",
    "currency",
    "payment_method",
    "payment_status",
    "status",
    "index",
];

pub fn validate_all(config: &MockConfig, source: &str, out: &mut Vec<Diagnostic>) {
    for (i, route) in config.routes.iter().enumerate() {
        let body_fields: Vec<String> = route
            .request
            .as_ref()
            .and_then(|request| request.body.as_ref())
            .map(|body| body.keys().cloned().collect())
            .unwrap_or_default();

        walk_value(
            &route.response.body,
            &format!("routes[{i}].response.body"),
            source,
            &route.path,
            &body_fields,
            out,
        );

        if let Some(headers) = &route.response.headers {
            for (key, value) in headers {
                if let Some(text) = value.as_str() {
                    check_string_placeholders(
                        text,
                        &format!("routes[{i}].response.headers.{key}"),
                        source,
                        &route.path,
                        &body_fields,
                        out,
                    );
                }
            }
        }
    }
}

fn walk_value(
    value: &Value,
    path: &str,
    source: &str,
    route_path: &str,
    body_fields: &[String],
    out: &mut Vec<Diagnostic>,
) {
    match value {
        Value::String(text) => {
            check_string_placeholders(text, path, source, route_path, body_fields, out);
        }
        Value::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                walk_value(
                    item,
                    &format!("{path}[{index}]"),
                    source,
                    route_path,
                    body_fields,
                    out,
                );
            }
        }
        Value::Object(map) => {
            if is_repeat_spec(map) {
                validate_repeat_spec(map, path, source, out);
            }

            for (key, item) in map {
                walk_value(
                    item,
                    &format!("{path}.{key}"),
                    source,
                    route_path,
                    body_fields,
                    out,
                );
            }
        }
        _ => {}
    }
}

fn is_repeat_spec(map: &serde_json::Map<String, Value>) -> bool {
    map.contains_key("repeat") && map.contains_key("item")
}

fn validate_repeat_spec(
    map: &serde_json::Map<String, Value>,
    path: &str,
    source: &str,
    out: &mut Vec<Diagnostic>,
) {
    if map.contains_key("item") && !map.contains_key("repeat") {
        let (line, column) = find_text(source, "\"item\"").unwrap_or((1, 1));
        out.push(Diagnostic::error(
            "INVALID_REPEAT_SPEC",
            path,
            line,
            column,
            "repeat spec must include both 'repeat' and 'item' fields",
            Some("Example: { \"repeat\": 3, \"item\": { \"id\": \"{{uuid}}\" } }".into()),
        ));
    }
}

fn check_string_placeholders(
    text: &str,
    path: &str,
    source: &str,
    route_path: &str,
    body_fields: &[String],
    out: &mut Vec<Diagnostic>,
) {
    if !text.contains("{{") {
        return;
    }

    if text.starts_with("{{") && text.ends_with("}}") && !text[2..text.len() - 2].contains("{{") {
        if let Some((code, message, hint)) = check_placeholder(text, route_path, body_fields) {
            let (line, column) = find_text(source, text).unwrap_or((1, 1));
            let severity = if code == "UNRESOLVED_BODY_FIELD" {
                super::diagnostics::Severity::Warning
            } else {
                super::diagnostics::Severity::Error
            };

            out.push(Diagnostic {
                severity,
                code,
                path: path.into(),
                line,
                column,
                message,
                hint,
            });
        }
        return;
    }

    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            let (line, column) = find_text(source, "{{").unwrap_or((1, 1));
            out.push(Diagnostic::error(
                "UNCLOSED_PLACEHOLDER",
                path,
                line,
                column,
                "placeholder opened with '{{' but not closed with '}}'",
                None,
            ));
            break;
        };

        let token = format!("{{{{{}}}}}", &after[..end]);
        if let Some((code, message, hint)) = check_placeholder(&token, route_path, body_fields) {
            let (line, column) = find_text(source, &token).unwrap_or((1, 1));
            let severity = if code == "UNRESOLVED_BODY_FIELD" {
                super::diagnostics::Severity::Warning
            } else {
                super::diagnostics::Severity::Error
            };

            out.push(Diagnostic {
                severity,
                code,
                path: path.into(),
                line,
                column,
                message,
                hint,
            });
        }

        rest = &after[end + 2..];
    }
}

fn check_placeholder(
    token: &str,
    route_path: &str,
    body_fields: &[String],
) -> Option<(String, String, Option<String>)> {
    let key = token.trim_start_matches("{{").trim_end_matches("}}");

    if STATIC_PLACEHOLDERS.contains(&key) {
        return None;
    }

    if let Some(param) = key.strip_prefix("param:") {
        if param.is_empty() {
            return Some((
                "INVALID_PARAM_PLACEHOLDER".into(),
                "param placeholder must include a name: {{param:id}}".into(),
                None,
            ));
        }

        if !route_path.contains(&format!(":{param}")) {
            let params = extract_path_params(route_path);
            return Some((
                "UNRESOLVED_PARAM".into(),
                format!("{{{{param:{param}}}}} is not defined in path '{route_path}'"),
                Some(format!(
                    "Available path params: {}",
                    if params.is_empty() {
                        "(none)".into()
                    } else {
                        params.join(", ")
                    }
                )),
            ));
        }
        return None;
    }

    if let Some(field) = key.strip_prefix("body:") {
        if field.is_empty() {
            return Some((
                "INVALID_BODY_PLACEHOLDER".into(),
                "body placeholder must include a field name: {{body:email}}".into(),
                None,
            ));
        }

        if !body_fields.iter().any(|name| name == field) {
            return Some((
                "UNRESOLVED_BODY_FIELD".into(),
                format!("{{{{body:{field}}}}} is not declared in request.body"),
                Some(format!(
                    "Declared body fields: {}",
                    if body_fields.is_empty() {
                        "(none)".into()
                    } else {
                        body_fields.join(", ")
                    }
                )),
            ));
        }
        return None;
    }

    if let Some(offset) = key.strip_prefix("index:") {
        if offset.parse::<u64>().is_err() {
            return Some((
                "INVALID_INDEX".into(),
                format!("{{{{index:{offset}}}}} must use a numeric offset"),
                None,
            ));
        }
        return None;
    }

    if let Some(file) = key.strip_prefix("csv_count:") {
        if file.is_empty() {
            return Some((
                "INVALID_CSV_COUNT".into(),
                "csv_count placeholder must include a filename: {{csv_count:users.csv}}".into(),
                None,
            ));
        }
        return None;
    }

    if let Some(rest) = key.strip_prefix("csv:") {
        let parts: Vec<&str> = rest.split(':').collect();
        if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Some((
                "INVALID_CSV_PLACEHOLDER".into(),
                "csv placeholder must use format {{csv:filename.csv:column}}".into(),
                None,
            ));
        }
        return None;
    }

    Some((
        "UNKNOWN_PLACEHOLDER".into(),
        format!("unknown placeholder '{token}'"),
        Some(format!(
            "Supported: {}, param:<name>, body:<field>, index[:offset], csv:<file>:<column>, csv_count:<file>",
            STATIC_PLACEHOLDERS.join(", ")
        )),
    ))
}

fn extract_path_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|segment| segment.strip_prefix(':'))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_placeholders_from_mixed_string() {
        assert!(check_placeholder("{{uuid}}", "/users/:id", &[]).is_none());
        assert!(check_placeholder("{{not_real}}", "/users/:id", &[]).is_some());
    }

    #[test]
    fn validates_param_against_path() {
        assert!(check_placeholder("{{param:id}}", "/users/:id", &[]).is_none());
        assert!(check_placeholder("{{param:userId}}", "/users/:id", &[]).is_some());
    }
}
