//! Parser for Cargo.lock files

use std::path::Path;
use serde::Deserialize;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for Cargo.lock lockfiles
pub struct CargoLockParser;

#[derive(Debug, Deserialize)]
struct CargoLock {
    #[serde(default)]
    package: Vec<Package>,
}

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
}

impl Parser for CargoLockParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let cargo_lock: CargoLock = toml::from_str(content)
            .map_err(|e| ScanError::toml_error(file_path.to_path_buf(), e))?;
        
        let mut records = Vec::new();
        
        for package in cargo_lock.package {
            records.push(DependencyRecord {
                name: package.name,
                version: package.version,
                source_file: file_path.to_path_buf(),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Rust,
                file_type: FileType::Lockfile,
            });
        }
        
        Ok(records)
    }
    
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Rust
    }
    
    fn file_type(&self) -> FileType {
        FileType::Lockfile
    }
    
    fn filename(&self) -> &str {
        "Cargo.lock"
    }
}
