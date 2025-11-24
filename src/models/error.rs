//! Error types for the scanner

use std::path::PathBuf;
use thiserror::Error;

/// Scanner error types
#[derive(Debug, Error)]
pub enum ScanError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Parse error in a specific file
    #[error("Parse error in {file:?}: {message}")]
    Parse {
        file: PathBuf,
        message: String,
    },
    
    /// Unsupported file format
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    
    /// Version parse error
    #[error("Version parse error: {0}")]
    VersionParse(String),
    
    /// JSON parsing error
    #[error("JSON parse error in {file:?}: {source}")]
    Json {
        file: PathBuf,
        source: serde_json::Error,
    },
    
    /// TOML parsing error
    #[error("TOML parse error in {file:?}: {source}")]
    Toml {
        file: PathBuf,
        source: toml::de::Error,
    },
    
    /// YAML parsing error
    #[error("YAML parse error in {file:?}: {source}")]
    Yaml {
        file: PathBuf,
        source: serde_yaml::Error,
    },
}

impl ScanError {
    /// Create a parse error
    pub fn parse_error(file: PathBuf, message: impl Into<String>) -> Self {
        ScanError::Parse {
            file,
            message: message.into(),
        }
    }
    
    /// Create a JSON error
    pub fn json_error(file: PathBuf, source: serde_json::Error) -> Self {
        ScanError::Json { file, source }
    }
    
    /// Create a TOML error
    pub fn toml_error(file: PathBuf, source: toml::de::Error) -> Self {
        ScanError::Toml { file, source }
    }
    
    /// Create a YAML error
    pub fn yaml_error(file: PathBuf, source: serde_yaml::Error) -> Self {
        ScanError::Yaml { file, source }
    }
}
