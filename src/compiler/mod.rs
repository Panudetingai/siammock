mod diagnostics;
mod placeholders;
mod routes;
mod span;

pub use diagnostics::{CompileResult, Diagnostic, Severity};

use crate::config::schema::MockConfig;

pub fn validate(source: &str) -> CompileResult {
    let mut diagnostics = Vec::new();

    let value: serde_json::Value = match serde_json::from_str(source) {
        Ok(value) => value,
        Err(err) => {
            let (line, column) = span::json_error_location(source, &err);
            diagnostics.push(Diagnostic::error(
                "JSON_SYNTAX",
                "$",
                line,
                column,
                format!("JSON syntax error: {err}"),
                Some("Check quotes, commas, and brackets".into()),
            ));
            return CompileResult {
                valid: false,
                diagnostics,
            };
        }
    };

    let config: MockConfig = match serde_json::from_value(value) {
        Ok(config) => config,
        Err(err) => {
            let (line, column) = span::json_error_location(source, &err);
            diagnostics.push(Diagnostic::error(
                "SCHEMA_MISMATCH",
                "$",
                line,
                column,
                format!("JSON structure does not match SiamMock schema: {err}"),
                Some(
                    "Top-level object must include routes[]; each route needs path, method, response"
                        .into(),
                ),
            ));
            return CompileResult {
                valid: false,
                diagnostics,
            };
        }
    };

    routes::validate_routes(&config, source, &mut diagnostics);
    placeholders::validate_all(&config, source, &mut diagnostics);

    let valid = !diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error);

    CompileResult {
        valid,
        diagnostics,
    }
}

pub fn validate_with_path(source: &str, file_path: &str) -> CompileResult {
    let mut result = validate(source);
    for diagnostic in &mut result.diagnostics {
        diagnostic.path = format!("{file_path}:{}", diagnostic.path);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let source = std::fs::read_to_string("mock/default.json").expect("default config");
        let result = validate(&source);
        assert!(result.valid, "{result:?}");
    }

    #[test]
    fn rejects_unknown_placeholder() {
        let source = r#"{
            "routes": [{
                "path": "/test",
                "method": "GET",
                "response": {
                    "status": 200,
                    "body": { "id": "{{not_real}}" }
                }
            }]
        }"#;

        let result = validate(source);
        assert!(!result.valid);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "UNKNOWN_PLACEHOLDER"));
    }

    #[test]
    fn rejects_invalid_http_method() {
        let source = r#"{
            "routes": [{
                "path": "/test",
                "method": "FOO",
                "response": { "status": 200, "body": {} }
            }]
        }"#;

        let result = validate(source);
        assert!(!result.valid);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "INVALID_HTTP_METHOD"));
    }

    #[test]
    fn rejects_unresolved_param() {
        let source = r#"{
            "routes": [{
                "path": "/users/:id",
                "method": "GET",
                "response": {
                    "status": 200,
                    "body": { "id": "{{param:userId}}" }
                }
            }]
        }"#;

        let result = validate(source);
        assert!(!result.valid);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "UNRESOLVED_PARAM"));
    }

    #[test]
    fn rejects_invalid_json_syntax() {
        let source = r#"{ "routes": [ { "path": "/broken" "#;
        let result = validate(source);
        assert!(!result.valid);
        assert!(result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "JSON_SYNTAX"));
    }
}
