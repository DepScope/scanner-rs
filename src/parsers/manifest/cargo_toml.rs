//! Parser for Cargo.toml files

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for Cargo.toml manifest files
pub struct CargoTomlParser;

#[derive(Debug, Deserialize)]
struct CargoToml {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, toml::Value>,
    #[serde(default, rename = "build-dependencies")]
    build_dependencies: HashMap<String, toml::Value>,
}

impl Parser for CargoTomlParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let cargo_toml: CargoToml = toml::from_str(content)
            .map_err(|e| ScanError::toml_error(file_path.to_path_buf(), e))?;

        let mut records = Vec::new();

        // Parse runtime dependencies
        for (name, value) in cargo_toml.dependencies {
            let version = extract_cargo_version(&value);
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Rust,
                file_type: FileType::Manifest,
            });
        }

        // Parse dev dependencies
        for (name, value) in cargo_toml.dev_dependencies {
            let version = extract_cargo_version(&value);
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Development,
                ecosystem: Ecosystem::Rust,
                file_type: FileType::Manifest,
            });
        }

        // Parse build dependencies
        for (name, value) in cargo_toml.build_dependencies {
            let version = extract_cargo_version(&value);
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Build,
                ecosystem: Ecosystem::Rust,
                file_type: FileType::Manifest,
            });
        }

        Ok(records)
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }

    fn file_type(&self) -> FileType {
        FileType::Manifest
    }

    fn filename(&self) -> &str {
        "Cargo.toml"
    }
}

/// Extract version from Cargo dependency value
fn extract_cargo_version(value: &toml::Value) -> String {
    match value {
        toml::Value::String(s) => s.clone(),
        toml::Value::Table(t) => {
            if let Some(toml::Value::String(v)) = t.get("version") {
                v.clone()
            } else {
                "*".to_string()
            }
        }
        _ => "*".to_string(),
    }
}
