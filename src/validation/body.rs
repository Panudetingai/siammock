use std::collections::HashMap;

use serde_json::Value;

pub fn validate_body(
    schema: &HashMap<String, Value>,
    body: &Value,
) -> Result<(), HashMap<String, String>> {
    let object = match body.as_object() {
        Some(obj) => obj,
        None => {
            let mut errors = HashMap::new();
            errors.insert("body".into(), "expected JSON object".into());
            return Err(errors);
        }
    };

    let mut errors = HashMap::new();

    for (field, expected) in schema {
        let Some(expected_type) = expected.as_str() else {
            continue;
        };

        if !is_validatable_type(expected_type) {
            continue;
        }

        match object.get(field) {
            None => {
                if is_required_field(expected_type) {
                    errors.insert(field.clone(), "missing required field".into());
                }
            }
            Some(value) => {
                if let Some(message) = type_mismatch(expected_type, value) {
                    errors.insert(field.clone(), message);
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn is_validatable_type(expected: &str) -> bool {
    matches!(
        normalize_type(expected),
        Some(
            "string" | "number" | "boolean" | "array" | "object" | "string[]" | "number[]"
                | "boolean[]"
        )
    ) || expected.contains("(required")
}

fn is_required_field(expected: &str) -> bool {
    matches!(
        normalize_type(expected),
        Some("string" | "number" | "boolean" | "array" | "object" | "string[]" | "number[]" | "boolean[]")
    ) || expected.contains("(required")
}

fn normalize_type(expected: &str) -> Option<&str> {
    let base = expected.split_whitespace().next()?.trim();
    match base {
        "string" | "number" | "boolean" | "array" | "object" | "string[]" | "number[]" | "boolean[]" => {
            Some(base)
        }
        _ => None,
    }
}

fn type_mismatch(expected: &str, actual: &Value) -> Option<String> {
    let kind = normalize_type(expected)?;

    let matches = match kind {
        "string" => actual.is_string(),
        "number" => actual.is_number(),
        "boolean" => actual.is_boolean(),
        "array" => actual.is_array(),
        "object" => actual.is_object(),
        "string[]" => actual.as_array().is_some_and(|items| items.iter().all(Value::is_string)),
        "number[]" => actual.as_array().is_some_and(|items| items.iter().all(Value::is_number)),
        "boolean[]" => actual.as_array().is_some_and(|items| items.iter().all(Value::is_boolean)),
        _ => false,
    };

    if matches {
        None
    } else {
        Some(format!("expected {kind}, got {}", value_kind(actual)))
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn accepts_valid_body() {
        let schema = HashMap::from([
            ("name".into(), json!("string")),
            ("age".into(), json!("number")),
        ]);
        let body = json!({"name": "test", "age": 25});

        assert!(validate_body(&schema, &body).is_ok());
    }

    #[test]
    fn rejects_missing_field() {
        let schema = HashMap::from([("name".into(), json!("string"))]);
        let body = json!({});

        let err = validate_body(&schema, &body).unwrap_err();
        assert_eq!(err.get("name").map(String::as_str), Some("missing required field"));
    }

    #[test]
    fn skips_example_values_in_schema() {
        let schema = HashMap::from([
            ("email".into(), json!("user@example.com")),
            ("password".into(), json!("SecretPassword123!")),
        ]);
        let body = json!({"email": "other@example.com", "password": "x"});

        assert!(validate_body(&schema, &body).is_ok());
    }

    #[test]
    fn validates_string_array_type() {
        let schema = HashMap::from([("tags".into(), json!("string[]"))]);
        let body = json!({"tags": ["a", "b"]});

        assert!(validate_body(&schema, &body).is_ok());
    }

    #[test]
    fn validates_descriptive_required_string() {
        let schema = HashMap::from([("first_name".into(), json!("string (required)"))]);
        let body = json!({});

        let err = validate_body(&schema, &body).unwrap_err();
        assert_eq!(
            err.get("first_name").map(String::as_str),
            Some("missing required field")
        );
    }
}
