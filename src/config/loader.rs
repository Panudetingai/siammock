use std::path::{Path, PathBuf};

use crate::compiler;
use crate::config::schema::MockConfig;

#[derive(Debug)]
pub enum ConfigError {
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    Parse {
        path: PathBuf,
        source: serde_json::Error,
    },
    Validation {
        path: PathBuf,
        diagnostics: compiler::CompileResult,
    },
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
                write!(
                    f,
                    "failed to parse config file {}: {source}",
                    path.display()
                )
            }
            Self::Validation { path, diagnostics } => {
                writeln!(f, "config validation failed for {}", path.display())?;
                for diagnostic in &diagnostics.diagnostics {
                    writeln!(
                        f,
                        "  [{}] {}:{} {} — {}",
                        match diagnostic.severity {
                            compiler::Severity::Error => "error",
                            compiler::Severity::Warning => "warning",
                            compiler::Severity::Info => "info",
                        },
                        diagnostic.line,
                        diagnostic.column,
                        diagnostic.code,
                        diagnostic.message
                    )?;
                    if let Some(hint) = &diagnostic.hint {
                        writeln!(f, "    hint: {hint}")?;
                    }
                }
                Ok(())
            }
            Self::NotFound(path) => write!(f, "config path not found: {path}"),
            Self::NoConfigFiles(path) => {
                write!(f, "no .json or .jsonsi config files found in: {path}")
            }
        }
    }
}

impl std::error::Error for ConfigError {}

pub fn is_config_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("jsonsi"))
}

pub fn load_configs(source: &str) -> Result<(MockConfig, Vec<PathBuf>), ConfigError> {
    let files = resolve_config_files(source)?;
    let mut routes = Vec::new();

    for file in &files {
        let config = load_config_file(file)?;
        routes.extend(config.routes);
    }

    Ok((MockConfig { routes }, files))
}

pub fn resolve_config_files(source: &str) -> Result<Vec<PathBuf>, ConfigError> {
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
            .filter(|file| file.is_file() && is_config_file(file))
            .collect::<Vec<_>>();

        files.sort();

        if files.is_empty() {
            return Err(ConfigError::NoConfigFiles(source.to_string()));
        }

        return Ok(files);
    }

    if path.is_file() {
        if !is_config_file(path) {
            return Err(ConfigError::NotFound(format!(
                "{} is not a SiamMock config file (expected .json or .jsonsi)",
                source
            )));
        }
        return Ok(vec![path.to_path_buf()]);
    }

    Err(ConfigError::NotFound(source.to_string()))
}

fn load_config_file(path: &Path) -> Result<MockConfig, ConfigError> {
    let content = std::fs::read_to_string(path).map_err(|source| ConfigError::Read {
        path: path.to_path_buf(),
        source,
    })?;

    let validation = compiler::validate_with_path(&content, &path.display().to_string());
    if !validation.valid {
        return Err(ConfigError::Validation {
            path: path.to_path_buf(),
            diagnostics: validation,
        });
    }

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
    fn loads_jsonsi_file() {
        let jsonsi = Path::new("mock/example.jsonsi");
        if !jsonsi.exists() {
            return;
        }

        let (config, files) = load_configs("mock/example.jsonsi").expect("jsonsi should load");
        assert!(
            files
                .iter()
                .any(|file| file.extension().is_some_and(|ext| ext == "jsonsi"))
        );
        assert!(!config.routes.is_empty());
    }

    #[test]
    fn rejects_non_config_extension() {
        assert!(load_configs("Cargo.toml").is_err());
    }

    #[test]
    fn loads_comma_separated_files() {
        let (config, files) =
            load_configs("mock/default.json,mock/payment.json").expect("files should load");
        assert_eq!(files.len(), 2);
        assert!(
            config
                .routes
                .iter()
                .any(|route| route.path == "/api/payment")
        );
    }
}
