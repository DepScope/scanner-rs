//! Parser for package-lock.json files

use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for package-lock.json lockfiles
pub struct PackageLockJsonParser;

#[derive(Debug, Deserialize)]
struct PackageLockJson {
    #[serde(default)]
    dependencies: HashMap<String, DependencyEntry>,
    #[serde(default)]
    packages: HashMap<String, PackageEntry>,
}

#[derive(Debug, Deserialize)]
struct DependencyEntry {
    version: String,
    #[serde(default)]
    dependencies: HashMap<String, DependencyEntry>,
}

#[derive(Debug, Deserialize)]
struct PackageEntry {
    #[serde(default)]
    version: Option<String>,
}

impl Parser for PackageLockJsonParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let package_lock: PackageLockJson = serde_json::from_str(content)
            .map_err(|e| ScanError::json_error(file_path.to_path_buf(), e))?;

        let mut records = Vec::new();

        // Parse from dependencies section (v1 format)
        for (name, entry) in &package_lock.dependencies {
            records.push(DependencyRecord {
                name: name.clone(),
                version: entry.version.clone(),
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Lockfile,
            });

            // Recursively parse nested dependencies
            parse_nested_dependencies(&entry.dependencies, file_path, &mut records);
        }

        // Parse from packages section (v2/v3 format)
        for (key, entry) in &package_lock.packages {
            // Skip the root package (empty key or just "")
            if key.is_empty() || key.is_empty() {
                continue;
            }

            if let Some(version) = &entry.version {
                // Extract package name from key (e.g., "node_modules/react" -> "react")
                let name = if key.starts_with("node_modules/") {
                    key.strip_prefix("node_modules/").unwrap_or(key)
                } else {
                    key.as_str()
                };

                // Only add if not already present from dependencies section
                if !records
                    .iter()
                    .any(|r| r.name == name && r.version == *version)
                {
                    records.push(DependencyRecord {
                        name: name.to_string(),
                        version: version.clone(),
                        source_file: file_path.to_path_buf(),
                        dep_type: DependencyType::Runtime,
                        ecosystem: Ecosystem::Node,
                        file_type: FileType::Lockfile,
                    });
                }
            }
        }

        Ok(records)
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Node
    }

    fn file_type(&self) -> FileType {
        FileType::Lockfile
    }

    fn filename(&self) -> &str {
        "package-lock.json"
    }
}

fn parse_nested_dependencies(
    dependencies: &HashMap<String, DependencyEntry>,
    file_path: &Path,
    records: &mut Vec<DependencyRecord>,
) {
    for (name, entry) in dependencies {
        // Only add if not already present
        if !records
            .iter()
            .any(|r| r.name == *name && r.version == entry.version)
        {
            records.push(DependencyRecord {
                name: name.clone(),
                version: entry.version.clone(),
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Lockfile,
            });
        }

        // Recurse into nested dependencies
        parse_nested_dependencies(&entry.dependencies, file_path, records);
    }
}
