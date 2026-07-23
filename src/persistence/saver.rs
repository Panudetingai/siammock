use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use chrono::Utc;
use serde_json::{Value, json};

use crate::config::schema::SaveSpec;

#[derive(Debug)]
pub enum SaveError {
    InvalidFormat(String),
    InvalidMode(String),
    Write(String),
    Read(String),
    Parse(String),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "invalid format: {msg}"),
            Self::InvalidMode(msg) => write!(f, "invalid mode: {msg}"),
            Self::Write(msg) => write!(f, "write error: {msg}"),
            Self::Read(msg) => write!(f, "read error: {msg}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for SaveError {}

pub fn save_request(
    spec: &SaveSpec,
    body: &Value,
    _params: &HashMap<String, String>,
    data_dir: &str,
) -> Result<(), SaveError> {
    let json_path = resolve_path(data_dir, &json_file_name(&spec.file));
    let csv_path = resolve_path(data_dir, &csv_file_name(&spec.file));

    match spec.format.as_str() {
        "json" => append_json(&json_path, body, &spec.mode),
        "csv" => append_csv(&csv_path, body, spec),
        "both" => {
            append_json(&json_path, body, &spec.mode)?;
            append_csv(&csv_path, body, spec)
        }
        other => Err(SaveError::InvalidFormat(other.to_string())),
    }
}

fn json_file_name(file: &str) -> String {
    with_extension(file, "json")
}

fn csv_file_name(file: &str) -> String {
    with_extension(file, "csv")
}

fn with_extension(file: &str, extension: &str) -> String {
    let path = Path::new(file);
    let parent = path.parent().unwrap_or(Path::new(""));
    let stem = path
        .file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or(file);

    if parent.as_os_str().is_empty() {
        format!("{stem}.{extension}")
    } else {
        parent
            .join(format!("{stem}.{extension}"))
            .to_string_lossy()
            .into_owned()
    }
}

fn resolve_path(data_dir: &str, file: &str) -> PathBuf {
    let base = Path::new(data_dir);
    let path = base.join(file);

    if path.strip_prefix(base).is_err() {
        panic!("invalid file path: {}", path.display());
    }

    path
}

fn append_json(path: &Path, body: &Value, mode: &str) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| SaveError::Write(e.to_string()))?;
    }

    let mut record = body.clone();
    if let Some(obj) = record.as_object_mut() {
        obj.insert("_saved_at".into(), json!(Utc::now().to_rfc3339()));
    }

    match mode {
        "append" => {
            let mut items: Vec<Value> = if path.exists() {
                let content =
                    fs::read_to_string(path).map_err(|e| SaveError::Read(e.to_string()))?;
                if content.trim().is_empty() {
                    Vec::new()
                } else {
                    serde_json::from_str(&content).map_err(|e| SaveError::Parse(e.to_string()))?
                }
            } else {
                Vec::new()
            };

            items.push(record);

            let json = serde_json::to_string_pretty(&items)
                .map_err(|e| SaveError::Write(e.to_string()))?;
            fs::write(path, json).map_err(|e| SaveError::Write(e.to_string()))?;
        }
        "overwrite" => {
            let json = serde_json::to_string_pretty(&[record])
                .map_err(|e| SaveError::Write(e.to_string()))?;
            fs::write(path, json).map_err(|e| SaveError::Write(e.to_string()))?;
        }
        other => return Err(SaveError::InvalidMode(other.to_string())),
    }

    Ok(())
}

fn append_csv(path: &Path, body: &Value, spec: &SaveSpec) -> Result<(), SaveError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| SaveError::Write(e.to_string()))?;
    }

    let target = extract_save_target(body, spec.path.as_deref())?;
    let rows = collect_csv_rows(&target)?;

    if rows.is_empty() {
        return Ok(());
    }

    let columns = resolve_columns(spec, rows[0])?;

    match spec.mode.as_str() {
        "append" => append_csv_rows(path, &columns, rows),
        "overwrite" => overwrite_csv_rows(path, &columns, rows),
        other => Err(SaveError::InvalidMode(other.to_string())),
    }
}

fn extract_save_target(body: &Value, path: Option<&str>) -> Result<Value, SaveError> {
    match path {
        None => Ok(body.clone()),
        Some(key) => body.get(key).cloned().ok_or_else(|| {
            SaveError::Parse(format!("path '{key}' not found in response"))
        }),
    }
}

fn collect_csv_rows(target: &Value) -> Result<Vec<&Value>, SaveError> {
    match target {
        Value::Array(items) => Ok(items.iter().collect()),
        Value::Object(_) => Ok(vec![target]),
        _ => Err(SaveError::Parse(
            "csv save requires a json object or array".into(),
        )),
    }
}

fn resolve_columns(spec: &SaveSpec, sample: &Value) -> Result<Vec<String>, SaveError> {
    if let Some(columns) = &spec.columns {
        return Ok(columns.clone());
    }

    sample
        .as_object()
        .map(|obj| obj.keys().cloned().collect())
        .ok_or_else(|| SaveError::Parse("csv row must be a json object".into()))
}

fn append_csv_rows(path: &Path, columns: &[String], rows: Vec<&Value>) -> Result<(), SaveError> {
    let file_exists = path.exists();
    let file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| SaveError::Write(e.to_string()))?;

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    if !file_exists {
        write_csv_header(&mut writer, columns)?;
    }

    for row in rows {
        write_csv_record(&mut writer, columns, row)?;
    }

    writer.flush().map_err(|e| SaveError::Write(e.to_string()))
}

fn overwrite_csv_rows(path: &Path, columns: &[String], rows: Vec<&Value>) -> Result<(), SaveError> {
    let file = fs::File::create(path).map_err(|e| SaveError::Write(e.to_string()))?;
    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .from_writer(file);

    write_csv_header(&mut writer, columns)?;

    for row in rows {
        write_csv_record(&mut writer, columns, row)?;
    }

    writer.flush().map_err(|e| SaveError::Write(e.to_string()))
}

fn write_csv_header(
    writer: &mut csv::Writer<impl Write>,
    columns: &[String],
) -> Result<(), SaveError> {
    let mut headers = columns.to_vec();
    headers.push("_saved_at".into());
    writer
        .write_record(&headers)
        .map_err(|e| SaveError::Write(e.to_string()))
}

fn write_csv_record(
    writer: &mut csv::Writer<impl Write>,
    columns: &[String],
    row: &Value,
) -> Result<(), SaveError> {
    let obj = row
        .as_object()
        .ok_or_else(|| SaveError::Parse("csv row must be a json object".into()))?;

    let saved_at = Utc::now().to_rfc3339();
    let record: Vec<String> = columns
        .iter()
        .map(|col| value_to_csv_cell(obj.get(col)))
        .chain(std::iter::once(saved_at))
        .collect();

    writer
        .write_record(&record)
        .map_err(|e| SaveError::Write(e.to_string()))
}

fn value_to_csv_cell(value: Option<&Value>) -> String {
    match value {
        None | Some(Value::Null) => String::new(),
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::Bool(flag)) => flag.to_string(),
        Some(other) => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_csv_path(name: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("siammock-{name}-{stamp}.csv"))
    }

    #[test]
    fn saves_rows_from_nested_data_path() {
        let path = temp_csv_path("nested-data");
        let body = json!({
            "success": true,
            "data": [
                {
                    "id": "1",
                    "first_name": "A",
                    "email": "a@example.com",
                    "email_verified": true
                },
                {
                    "id": "2",
                    "first_name": "B",
                    "email": "b@example.com",
                    "email_verified": false
                }
            ]
        });
        let spec = SaveSpec {
            format: "csv".into(),
            file: path.to_string_lossy().into(),
            path: Some("data".into()),
            mode: "overwrite".into(),
            columns: Some(vec![
                "id".into(),
                "first_name".into(),
                "email".into(),
                "email_verified".into(),
            ]),
        };

        append_csv(&path, &body, &spec).expect("csv save should succeed");

        let content = fs::read_to_string(&path).expect("csv file should exist");
        assert!(content.contains("1,A,a@example.com,true"));
        assert!(content.contains("2,B,b@example.com,false"));

        let _ = fs::remove_file(path);
    }

    #[test]
    fn saves_single_object_without_path() {
        let path = temp_csv_path("single-object");
        let body = json!({
            "id": "42",
            "email": "user@example.com"
        });
        let spec = SaveSpec {
            format: "csv".into(),
            file: path.to_string_lossy().into(),
            path: None,
            mode: "overwrite".into(),
            columns: Some(vec!["id".into(), "email".into()]),
        };

        append_csv(&path, &body, &spec).expect("csv save should succeed");

        let content = fs::read_to_string(&path).expect("csv file should exist");
        assert!(content.contains("42,user@example.com"));

        let _ = fs::remove_file(path);
    }
}