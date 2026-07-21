use std::path::{Path, PathBuf};

use crate::config::schema::MockConfig;

#[derive(Debug)]
pub enum ConfigError {
    Read { path: PathBuf, source: std::io::Error },
    Parse { path: PathBuf, source: serde_json::Error },
    NotFound(String),
    NoConfigFiles(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Read { path, source } => {
                write!(f, "failed to read config file {}: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(f, "failed to parse config file {}: {source}", path.display())
            }
            Self::NotFound(path) => write!(f, "config path not found: {path}"),
            Self::NoConfigFiles(path) => {
                write!(f, "no .json config files found in: {path}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn load_configs(source: &str) -> Result<(MockConfig, Vec<PathBuf>), ConfigError> {
    let files = resolve_config_files(source)?;
    let mut routes = Vec::new();

    for file in &files {
        let config = load_config_file(file)?;
        routes.extend(config.routes);
    }

    Ok((MockConfig { routes }, files))
}

fn resolve_config_files(source: &str) -> Result<Vec<PathBuf>, ConfigError> {
    if source.contains(',') {
        let mut files = Vec::new();

        for part in source.split(',') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }
            files.extend(resolve_config_files(part)?);
        }

        if files.is_empty() {
            return Err(ConfigError::NoConfigFiles(source.to_string()));
        }

        return Ok(files);
    }

    let path = Path::new(source);

    if path.is_dir() {
        let mut files = std::fs::read_dir(path)
            .map_err(|source| ConfigError::Read {
                path: path.to_path_buf(),
                source,
            })?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|file| {
                file.is_file()
                    && file
                        .extension()
                        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
            })
            .collect::<Vec<_>>();

        files.sort();

        if files.is_empty() {
            return Err(ConfigError::NoConfigFiles(source.to_string()));
        }

        return Ok(files);
    }

    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    Err(ConfigError::NotFound(source.to_string()))
}

fn load_config_file(path: &Path) -> Result<MockConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    serde_json::from_str(&content).map_err(|source| ConfigError::Parse {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_all_json_files_from_directory() {
        let (config, files) = load_configs("mock").expect("mock directory should load");
        assert!(files.len() >= 2);
        assert!(config.routes.len() >= 5);
    }

    #[test]
    fn loads_comma_separated_files() {
        let (config, files) =
            load_configs("mock/default.json,mock/payment.json").expect("files should load");
        assert_eq!(files.len(), 2);
        assert!(config.routes.iter().any(|route| route.path == "/api/payment"));
    }
}
