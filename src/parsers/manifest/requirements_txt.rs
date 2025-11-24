//! Parser for requirements.txt files

use std::path::Path;

use crate::models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanError};
use crate::parsers::Parser;

/// Parser for requirements.txt manifest files
pub struct RequirementsTxtParser;

impl Parser for RequirementsTxtParser {
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError> {
        let mut records = Vec::new();

        for line in content.lines() {
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Skip -r, -c, --requirement, --constraint flags
            if line.starts_with("-r ")
                || line.starts_with("-c ")
                || line.starts_with("--requirement")
                || line.starts_with("--constraint")
            {
                continue;
            }

            // Skip editable installs and URLs for now
            if line.starts_with("-e ")
                || line.starts_with("git+")
                || line.starts_with("http://")
                || line.starts_with("https://")
            {
                continue;
            }

            // Parse package specification
            if let Some((name, version)) = parse_requirement_line(line) {
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

        Ok(records)
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Python
    }

    fn file_type(&self) -> FileType {
        FileType::Manifest
    }

    fn filename(&self) -> &str {
        "requirements.txt"
    }
}

/// Parse a single requirement line
fn parse_requirement_line(line: &str) -> Option<(String, String)> {
    // Remove inline comments first
    let line = if let Some(pos) = line.find('#') {
        line[..pos].trim()
    } else {
        line.trim()
    };

    // Parse version specifiers
    for op in &[">=", "<=", "==", "!=", "~=", ">", "<"] {
        if let Some(pos) = line.find(op) {
            let name_part = line[..pos].trim();
            let version = line[pos..].trim().to_string();

            // Remove extras from name (e.g., "requests[security]" -> "requests")
            let name = if let Some(bracket_pos) = name_part.find('[') {
                name_part[..bracket_pos].trim().to_string()
            } else {
                name_part.to_string()
            };

            return Some((name, version));
        }
    }

    // No version specified - remove extras from name
    if !line.is_empty() {
        let name = if let Some(pos) = line.find('[') {
            line[..pos].trim().to_string()
        } else {
            line.to_string()
        };
        Some((name, "*".to_string()))
    } else {
        None
    }
}
