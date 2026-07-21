use std::collections::HashMap;

use chrono::Utc;
use rand::Rng;
use serde::Serialize;
use serde_json::{Map, Value};
use uuid::Uuid;
use jsonwebtoken::{Header, EncodingKey, Algorithm};

use crate::data::CsvStore;
use crate::response::constants::{CURRENCIES, EN_NAMES, PAYMENT_METHODS, PAYMENT_STATUSES, STATUS, THAI_NAMES};

const MAX_REPEAT: usize = 1_000;

pub struct RenderContext<'a> {
    pub params: &'a HashMap<String, String>,
    pub body: &'a Value,
    pub index: Option<usize>,
    pub csv: &'a CsvStore,
}

pub fn render_response(template: &Value, ctx: &RenderContext<'_>) -> Value {
    render_value(template, ctx)
}

fn render_value(value: &Value, ctx: &RenderContext<'_>) -> Value {
    match value {
        Value::String(text) => render_string(text, ctx),
        Value::Array(items) => Value::Array(items.iter().map(|item| render_value(item, ctx)).collect()),
        Value::Object(map) => render_object(map, ctx),
        other => other.clone(),
    }
}

fn render_object(map: &Map<String, Value>, ctx: &RenderContext<'_>) -> Value {
    if is_repeat_spec(map) {
        return render_repeat_spec(map, ctx);
    }

    let total = map.get("total").and_then(|value| resolve_repeat_count(value, ctx));

    let rendered = map
        .iter()
        .map(|(key, val)| {
            let rendered_val = match (total, val) {
                (Some(count), Value::Array(items)) if items.len() == 1 => {
                    expand_template(&items[0], count, ctx)
                }
                _ => render_value(val, ctx),
            };
            (key.clone(), rendered_val)
        })
        .collect::<Map<String, Value>>();

    Value::Object(rendered)
}

fn is_repeat_spec(map: &Map<String, Value>) -> bool {
    map.contains_key("repeat") && map.contains_key("item")
}

fn render_repeat_spec(map: &Map<String, Value>, ctx: &RenderContext<'_>) -> Value {
    let count = map
        .get("repeat")
        .and_then(|value| resolve_repeat_count(value, ctx))
        .unwrap_or(0)
        .min(MAX_REPEAT);

    let item = map.get("item").cloned().unwrap_or(Value::Null);
    expand_template(&item, count, ctx)
}

fn expand_template(template: &Value, count: usize, ctx: &RenderContext<'_>) -> Value {
    let capped = count.min(MAX_REPEAT);
    let items = (0..capped)
        .map(|index| {
            let item_ctx = RenderContext {
                params: ctx.params,
                body: ctx.body,
                index: Some(index),
                csv: ctx.csv,
            };
            render_value(template, &item_ctx)
        })
        .collect();

    Value::Array(items)
}

fn resolve_repeat_count(value: &Value, ctx: &RenderContext<'_>) -> Option<usize> {
    match value {
        Value::Number(number) => number.as_u64().map(|n| n as usize),
        Value::String(text) => {
            if is_placeholder(text) {
                return resolve_placeholder(text, ctx)
                    .as_u64()
                    .map(|n| n as usize);
            }
            text.parse().ok()
        }
        _ => None,
    }
}

fn render_string(text: &str, ctx: &RenderContext<'_>) -> Value {
    if is_placeholder(text) {
        return resolve_placeholder(text, ctx);
    }

    if text.contains("{{") {
        let mut rendered = text.to_string();
        while let Some(start) = rendered.find("{{") {
            let Some(end) = rendered[start..].find("}}") else {
                break;
            };
            let end = start + end + 2;
            let token = &rendered[start..end];
            let replacement = resolve_placeholder(token, ctx);
            let replacement_text = value_to_string(&replacement);
            rendered.replace_range(start..end, &replacement_text);
        }
        return Value::String(rendered);
    }

    Value::String(text.to_string())
}

fn is_placeholder(text: &str) -> bool {
    text.starts_with("{{") && text.ends_with("}}")
}

fn resolve_placeholder(token: &str, ctx: &RenderContext<'_>) -> Value {
    let key = token.trim_start_matches("{{").trim_end_matches("}}");

    match key {
        "uuid" => Value::String(Uuid::new_v4().to_string()),
        "timestamp" => Value::String(Utc::now().to_rfc3339()),
        "random_number" => {
            let number = rand::thread_rng().gen_range(1..=10_000);
            Value::Number(number.into())
        }
        "jwt_token" => Value::String(random_jwt_token()),
        "random_string" => Value::String(random_string()),
        "thai_name" => Value::String(random_thai_name()),
        "en_name" => Value::String(random_en_name()),
        "email" => Value::String(random_email()),
        "currency" => Value::String(random_currency()),
        "payment_method" => Value::String(random_payment_method()),
        "payment_status" => Value::String(random_payment_status()),
        "status" => Value::String(random_status()),
        key if key.starts_with("param:") => {
            let param = key.trim_start_matches("param:");
            ctx.params
                .get(param)
                .cloned()
                .map(Value::String)
                .unwrap_or(Value::Null)
        }
        key if key.starts_with("body:") => {
            let field = key.trim_start_matches("body:");
            ctx.body.get(field).cloned().unwrap_or(Value::Null)
        }
        "index" => ctx
            .index
            .map(|i| Value::Number((i as u64).into()))
            .unwrap_or(Value::Null),
        key if key.starts_with("index:") => {
            let offset = key
                .trim_start_matches("index:")
                .parse::<u64>()
                .unwrap_or(0);
            ctx.index
                .map(|i| Value::Number((i as u64 + offset).into()))
                .unwrap_or(Value::Null)
        }
        key if key.starts_with("csv_count:") => {
            let file = key.trim_start_matches("csv_count:");
            ctx.csv
                .row_count(file)
                .map(|count| Value::Number((count as u64).into()))
                .unwrap_or(Value::Null)
        }
        key if key.starts_with("csv:") => {
            let rest = key.trim_start_matches("csv:");
            let Some((file, column)) = rest.split_once(':') else {
                return Value::String(token.to_string());
            };

            ctx.csv
                .value(file, column, ctx.index)
                .map(Value::String)
                .unwrap_or(Value::Null)
        }
        _ => Value::String(token.to_string()),
    }
}

#[derive(Serialize)]
struct JwtClaims {
    sub: String,
    name: String,
    iat: i64,
    exp: i64,
}

fn random_jwt_token() -> String {
    let claims = JwtClaims {
        sub: random_string(),
        name: "JWT Token Example".to_string(),
        iat: Utc::now().timestamp(),
        exp: Utc::now().timestamp() + 3600,
    };
    let header = Header::new(Algorithm::HS256);
    let token = jsonwebtoken::encode(
        &header, 
        &claims, 
        &EncodingKey::from_secret(b"secret")
    ).unwrap();
    token
}

fn random_thai_name() -> String {
    let index = rand::thread_rng().gen_range(0..THAI_NAMES.len());
    THAI_NAMES[index].to_string()
}

fn random_en_name() -> String {
    let index = rand::thread_rng().gen_range(0..EN_NAMES.len());
    EN_NAMES[index].to_string()
}

fn random_string() -> String {
    let length = rand::thread_rng().gen_range(1..=10);
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = (0..length).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();
    chars.iter().collect()
}

fn random_currency() -> String {
    let index = rand::thread_rng().gen_range(0..CURRENCIES.len());
    CURRENCIES[index].to_string()
}

fn random_payment_method() -> String {
    let index = rand::thread_rng().gen_range(0..PAYMENT_METHODS.len());
    PAYMENT_METHODS[index].to_string()
}

fn random_payment_status() -> String {
    let index = rand::thread_rng().gen_range(0..PAYMENT_STATUSES.len());
    PAYMENT_STATUSES[index].to_string()
}

fn random_status() -> String {
    let index = rand::thread_rng().gen_range(0..STATUS.len());
    STATUS[index].to_string()
}


fn random_email() -> String {
    let number = rand::thread_rng().gen_range(1000..9999);
    format!("user{number}@example.com")
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(number) => number.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::CsvStore;
    use serde_json::json;

    fn empty_csv() -> CsvStore {
        CsvStore::default()
    }

    fn test_csv() -> CsvStore {
        CsvStore::load_from_dir("data").expect("test csv should load")
    }

    #[test]
    fn resolves_body_placeholder() {
        let csv = empty_csv();
        let params = HashMap::new();
        let body = json!({"name": "สมชาย"});
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: None,
            csv: &csv,
        };

        let rendered = render_response(&json!("{{body:name}}"), &ctx);
        assert_eq!(rendered, json!("สมชาย"));
    }

    #[test]
    fn resolves_param_placeholder() {
        let csv = empty_csv();
        let params = HashMap::from([("id".into(), "42".into())]);
        let body = json!(null);
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: None,
            csv: &csv,
        };

        let rendered = render_response(&json!("{{param:id}}"), &ctx);
        assert_eq!(rendered, json!("42"));
    }

    #[test]
    fn expands_array_from_total_field() {
        let csv = empty_csv();
        let params = HashMap::new();
        let body = json!(null);
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: None,
            csv: &csv,
        };

        let template = json!({
            "users": [{ "id": "{{uuid}}" }],
            "total": 3
        });

        let rendered = render_response(&template, &ctx);
        assert_eq!(rendered["total"], 3);
        assert_eq!(rendered["users"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn expands_repeat_spec() {
        let csv = empty_csv();
        let params = HashMap::new();
        let body = json!(null);
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: None,
            csv: &csv,
        };

        let template = json!({
            "users": {
                "repeat": 2,
                "item": { "no": "{{index:1}}" }
            }
        });

        let rendered = render_response(&template, &ctx);
        assert_eq!(rendered["users"][0]["no"], 1);
        assert_eq!(rendered["users"][1]["no"], 2);
    }

    #[test]
    fn resolves_csv_column_by_index() {
        let csv = test_csv();
        let params = HashMap::new();
        let body = json!(null);
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: Some(0),
            csv: &csv,
        };

        let rendered = render_response(&json!("{{csv:users.csv:email}}"), &ctx);
        assert_eq!(rendered, json!("somchai@example.com"));
    }

    #[test]
    fn expands_repeat_from_csv_row_count() {
        let csv = test_csv();
        let params = HashMap::new();
        let body = json!(null);
        let ctx = RenderContext {
            params: &params,
            body: &body,
            index: None,
            csv: &csv,
        };

        let template = json!({
            "users": {
                "repeat": "{{csv_count:users.csv}}",
                "item": {
                    "email": "{{csv:users.csv:email}}"
                }
            }
        });

        let rendered = render_response(&template, &ctx);
        assert_eq!(rendered["users"].as_array().unwrap().len(), 5);
    }
}
