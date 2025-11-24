//! Parser for yarn.lock files

use regex::Regex;
use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for yarn.lock lockfiles
pub struct YarnLockParser;

impl Parser for YarnLockParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let mut records = Vec::new();

        // Split content into records (separated by blank lines)
        let record_re = Regex::new(r"\n\s*\n").unwrap();
        let records_text: Vec<&str> = record_re.split(content).collect();

        // Regex to extract package name and version
        let name_re = Regex::new(r#"^["']?([^@\s"']+)@"#).unwrap();
        let version_re = Regex::new(r#"^\s*version\s+"([^"]+)""#).unwrap();

        for record in records_text {
            // Skip empty records
            if record.trim().is_empty() {
                continue;
            }

            // Extract package name from the first line
            let lines: Vec<&str> = record.lines().collect();
            if lines.is_empty() {
                continue;
            }

            let first_line = lines[0];
            let name = if let Some(cap) = name_re.captures(first_line) {
                cap[1].to_string()
            } else {
                continue;
            };

            // Extract version from the record
            let mut version = String::new();
            for line in &lines {
                if let Some(cap) = version_re.captures(line) {
                    version = cap[1].to_string();
                    break;
                }
            }

            if !version.is_empty() {
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
        "yarn.lock"
    }
}
