//! Parser for package.json files

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for package.json manifest files
pub struct PackageJsonParser;

#[derive(Debug, Deserialize)]
struct PackageJson {
    #[serde(default)]
    dependencies: HashMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    dev_dependencies: HashMap<String, String>,
    #[serde(default, rename = "peerDependencies")]
    peer_dependencies: HashMap<String, String>,
    #[serde(default, rename = "optionalDependencies")]
    optional_dependencies: HashMap<String, String>,
}

impl Parser for PackageJsonParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let package_json: PackageJson = serde_json::from_str(content)
            .map_err(|e| ScanError::json_error(file_path.to_path_buf(), e))?;

        let mut records = Vec::new();

        // Parse runtime dependencies
        for (name, version) in package_json.dependencies {
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Manifest,
            });
        }

        // Parse dev dependencies
        for (name, version) in package_json.dev_dependencies {
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Development,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Manifest,
            });
        }

        // Parse peer dependencies
        for (name, version) in package_json.peer_dependencies {
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Peer,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Manifest,
            });
        }

        // Parse optional dependencies
        for (name, version) in package_json.optional_dependencies {
            records.push(DependencyRecord {
                name,
                version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Optional,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Manifest,
            });
        }

        Ok(records)
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn file_type(&self) -> FileType {
        FileType::Manifest
    }

    fn filename(&self) -> &str {
        "package.json"
    }
}
