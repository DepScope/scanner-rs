//! Parser for pnpm-lock.yaml files

use regex::Regex;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for pnpm-lock.yaml lockfiles
pub struct PnpmLockParser;

impl Parser for PnpmLockParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let mut records = Vec::new();

        // Pattern 1: /package/1.2.3
        let pattern1 = Regex::new(r"/([^/@\s]+)/(\d+\.\d+\.\d+[^\s:]*)").unwrap();

        // Pattern 2: "package@1.2.3"
        let pattern2 = Regex::new(r#"["']([^@\s"']+)@(\d+\.\d+\.\d+[^\s"']*)"#).unwrap();

        // Extract using pattern 1
        for cap in pattern1.captures_iter(content) {
            let name = cap[1].to_string();
            let version = cap[2].to_string();

            // Avoid duplicates
            if !records
                .iter()
                .any(|r: &DependencyRecord| r.name == name && r.version == version)
            {
                records.push(DependencyRecord {
                    name,
                    version,
                    source_file: file_path.to_path_buf(),
                    dep_type: DependencyType::Runtime,
                    ecosystem: Ecosystem::Node,
                    file_type: FileType::Lockfile,
                });
            }
        }

        // Extract using pattern 2
        for cap in pattern2.captures_iter(content) {
            let name = cap[1].to_string();
            let version = cap[2].to_string();

            // Avoid duplicates
            if !records
                .iter()
                .any(|r: &DependencyRecord| r.name == name && r.version == version)
            {
                records.push(DependencyRecord {
                    name,
                    version,
                    source_file: file_path.to_path_buf(),
                    dep_type: DependencyType::Runtime,
                    ecosystem: Ecosystem::Node,
                    file_type: FileType::Lockfile,
                });
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
        "pnpm-lock.yaml"
    }
}
