//! Parser for pyproject.toml files

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for pyproject.toml manifest files
pub struct PyprojectTomlParser;

#[derive(Debug, Deserialize)]
struct PyprojectToml {
    #[serde(default)]
    project: Option<ProjectSection>,
    #[serde(default)]
    tool: Option<ToolSection>,
}

#[derive(Debug, Deserialize)]
struct ProjectSection {
    #[serde(default)]
    dependencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ToolSection {
    #[serde(default)]
    poetry: Option<PoetrySection>,
}

#[derive(Debug, Deserialize)]
struct PoetrySection {
    #[serde(default)]
    dependencies: HashMap<String, toml::Value>,
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: HashMap<String, toml::Value>,
}

impl Parser for PyprojectTomlParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let pyproject: PyprojectToml = toml::from_str(content)
            .map_err(|e| ScanError::toml_error(file_path.to_path_buf(), e))?;

        let mut records = Vec::new();

        // Parse PEP 621 dependencies (project.dependencies)
        if let Some(project) = pyproject.project {
            for dep_spec in project.dependencies {
                if let Some((name, version)) = parse_pep_508_dependency(&dep_spec) {
                    records.push(DependencyRecord {
                        name,
                        version,
                        source_file: file_path.to_path_buf(),
                        dep_type: DependencyType::Runtime,
                        ecosystem: Ecosystem::Python,
                        file_type: FileType::Manifest,
                    });
                }
            }
        }

        // Parse Poetry dependencies
        if let Some(tool) = pyproject.tool {
            if let Some(poetry) = tool.poetry {
                // Runtime dependencies
                for (name, value) in poetry.dependencies {
                    // Skip python itself
                    if name == "python" {
                        continue;
                    }

                    let version = extract_poetry_version(&value);
                    records.push(DependencyRecord {
                        name,
                        version,
                        source_file: file_path.to_path_buf(),
                        dep_type: DependencyType::Runtime,
                        ecosystem: Ecosystem::Python,
                        file_type: FileType::Manifest,
                    });
                }

                // Dev dependencies
                for (name, value) in poetry.dev_dependencies {
                    let version = extract_poetry_version(&value);
                    records.push(DependencyRecord {
                        name,
                        version,
                        source_file: file_path.to_path_buf(),
                        dep_type: DependencyType::Development,
                        ecosystem: Ecosystem::Python,
                        file_type: FileType::Manifest,
                    });
                }
            }
        }

        Ok(records)
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn file_type(&self) -> FileType {
        FileType::Manifest
    }

    fn filename(&self) -> &str {
        "pyproject.toml"
    }
}

/// Parse PEP 508 dependency specification (e.g., "requests>=2.28.0")
fn parse_pep_508_dependency(spec: &str) -> Option<(String, String)> {
    // Simple parsing: split on common operators
    let spec = spec.trim();

    for op in &[">=", "<=", "==", "!=", "~=", ">", "<"] {
        if let Some(pos) = spec.find(op) {
            let name = spec[..pos].trim().to_string();
            let version = spec[pos..].trim().to_string();
            return Some((name, version));
        }
    }

    // No version specified
    Some((spec.to_string(), "*".to_string()))
}

/// Extract version from Poetry dependency value
fn extract_poetry_version(value: &toml::Value) -> String {
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
