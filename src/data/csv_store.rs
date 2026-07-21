use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use rand::Rng;

#[derive(Debug, Clone, Default)]
pub struct CsvStore {
    tables: HashMap<String, CsvTable>,
}

#[derive(Debug, Clone)]
struct CsvTable {
    rows: Vec<HashMap<String, String>>,
}

#[derive(Debug)]
pub enum CsvError {
    Read { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, source: csv::Error },
}

impl std::fmt::Display for CsvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read csv {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(f, "failed to parse csv {}: {source}", path.display())
            }
        }
    }
}

impl std::error::Error for CsvError {}

impl CsvStore {
    pub fn load_from_dir(dir: &str) -> Result<Self, CsvError> {
        let path = Path::new(dir);

        if !path.exists() {
            return Ok(Self::default());
        }

        let mut tables = HashMap::new();

        for entry in fs::read_dir(path).map_err(|source| CsvError::Read {
            path: path.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| CsvError::Read {
                path: path.to_path_buf(),
                source,
            })?;
            let file_path = entry.path();

            if !file_path.is_file() {
                continue;
            }

            if !file_path
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("csv"))
            {
                continue;
            }

            let Some(file_name) = file_path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };

            tables.insert(file_name.to_string(), load_csv_file(&file_path)?);
        }

        Ok(Self { tables })
    }

    pub fn row_count(&self, file: &str) -> Option<usize> {
        self.tables.get(file).map(|table| table.rows.len())
    }

    pub fn value(&self, file: &str, column: &str, row_index: Option<usize>) -> Option<String> {
        let table = self.tables.get(file)?;
        if table.rows.is_empty() {
            return None;
        }

        let index = match row_index {
            Some(index) => index % table.rows.len(),
            None => rand::thread_rng().gen_range(0..table.rows.len()),
        };

        table.rows[index].get(column).cloned()
    }

    pub fn loaded_files(&self) -> Vec<&str> {
        let mut files: Vec<&str> = self.tables.keys().map(String::as_str).collect();
        files.sort_unstable();
        files
    }
}

fn load_csv_file(path: &Path) -> Result<CsvTable, CsvError> {
    let mut reader = csv::Reader::from_path(path).map_err(|source| CsvError::Parse {
        path: path.to_path_buf(),
        source,
    })?;

    let headers = reader
        .headers()
        .map_err(|source| CsvError::Parse {
            path: path.to_path_buf(),
            source,
        })?
        .iter()
        .map(str::to_string)
        .collect::<Vec<_>>();

    let mut rows = Vec::new();

    for record in reader.records() {
        let record = record.map_err(|source| CsvError::Parse {
            path: path.to_path_buf(),
            source,
        })?;

        let mut row = HashMap::new();
        for (header, value) in headers.iter().zip(record.iter()) {
            row.insert(header.clone(), value.to_string());
        }
        rows.push(row);
    }

    Ok(CsvTable { rows })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_csv_from_data_directory() {
        let store = CsvStore::load_from_dir("data").expect("data directory should load");
        assert!(store.row_count("users.csv").unwrap_or(0) >= 2);
        assert!(store.value("users.csv", "email", Some(0)).is_some());
    }
}
