//! Parser for Python package metadata (METADATA and PKG-INFO files)
//!
//! This module parses Python package metadata from installed packages:
//!
//! - **METADATA**: Modern format from .dist-info directories (PEP 566)
//! - **PKG-INFO**: Legacy format from .egg-info directories/files (PEP 314)
//!
//! Both formats contain package name, version, and dependency information.
//! The parser handles various dependency specification formats including:
//!
//! - Simple dependencies: `requests`
//! - Version constraints: `requests (>=2.0.0)`
//! - Complex constraints: `urllib3 (<3,>=1.21.1)`
//! - Extras/markers: `pytest ; extra == 'dev'` (extras are filtered out)
//!
//! # Example
//!
//! ```rust
//! use scanner::parsers::installed::{parse_metadata_file, parse_pkg_info_file};
//! use std::path::Path;
//!
//! // Parse METADATA from .dist-info
//! let metadata_path = Path::new("/site-packages/requests-2.31.0.dist-info/METADATA");
//! if let Ok(metadata) = parse_metadata_file(metadata_path) {
//!     println!("{} {}", metadata.name, metadata.version);
//!     for (dep_name, dep_version) in metadata.dependencies {
//!         println!("  → {} {}", dep_name, dep_version);
//!     }
//! }
//!
//! // Parse PKG-INFO from .egg-info
//! let pkg_info_path = Path::new("/site-packages/simplejson-3.19.1.egg-info/PKG-INFO");
//! if let Ok(metadata) = parse_pkg_info_file(pkg_info_path) {
//!     println!("{} {}", metadata.name, metadata.version);
//! }
//! ```

use crate::models::error::ScanError;
use std::fs;
use std::path::Path;

/// Parsed Python package metadata
#[derive(Debug, Clone)]
pub struct PythonMetadata {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Dependencies (from Requires-Dist)
    pub dependencies: Vec<(String, String)>, // (name, version_constraint)
}

/// Parse a METADATA file from a .dist-info directory
pub fn parse_metadata_file(path: &Path) -> Result<PythonMetadata, ScanError> {
    let content = fs::read_to_string(path).map_err(ScanError::Io)?;
    parse_metadata(&content, path)
}

/// Parse METADATA content
pub fn parse_metadata(content: &str, file_path: &Path) -> Result<PythonMetadata, ScanError> {
    let mut name = None;
    let mut version = None;
    let mut dependencies = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("Name:") {
            name = Some(line[5..].trim().to_string());
        } else if line.starts_with("Version:") {
            version = Some(line[8..].trim().to_string());
        } else if line.starts_with("Requires-Dist:") {
            // Parse dependency specification
            let dep_spec = line[14..].trim();
            if let Some((dep_name, dep_version)) = parse_requires_dist(dep_spec) {
                dependencies.push((dep_name, dep_version));
            }
        }
    }

    let name = name.ok_or_else(|| ScanError::Parse {
        file: file_path.to_path_buf(),
        message: "Missing 'Name' field in METADATA".to_string(),
    })?;

    let version = version.ok_or_else(|| ScanError::Parse {
        file: file_path.to_path_buf(),
        message: "Missing 'Version' field in METADATA".to_string(),
    })?;

    Ok(PythonMetadata {
        name,
        version,
        dependencies,
    })
}

/// Parse a PKG-INFO file from a .egg-info directory or file
pub fn parse_pkg_info_file(path: &Path) -> Result<PythonMetadata, ScanError> {
    let content = fs::read_to_string(path).map_err(ScanError::Io)?;
    parse_pkg_info(&content, path)
}

/// Parse PKG-INFO content (similar format to METADATA)
pub fn parse_pkg_info(content: &str, file_path: &Path) -> Result<PythonMetadata, ScanError> {
    // PKG-INFO has similar format to METADATA, but may use "Requires:" instead of "Requires-Dist:"
    let mut name = None;
    let mut version = None;
    let mut dependencies = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("Name:") {
            name = Some(line[5..].trim().to_string());
        } else if line.starts_with("Version:") {
            version = Some(line[8..].trim().to_string());
        } else if line.starts_with("Requires:") {
            // Simple dependency name (older format)
            let dep_name = line[9..].trim().to_string();
            if !dep_name.is_empty() {
                dependencies.push((dep_name, "*".to_string()));
            }
        } else if line.starts_with("Requires-Dist:") {
            // Modern format
            let dep_spec = line[14..].trim();
            if let Some((dep_name, dep_version)) = parse_requires_dist(dep_spec) {
                dependencies.push((dep_name, dep_version));
            }
        }
    }

    let name = name.ok_or_else(|| ScanError::Parse {
        file: file_path.to_path_buf(),
        message: "Missing 'Name' field in PKG-INFO".to_string(),
    })?;

    let version = version.ok_or_else(|| ScanError::Parse {
        file: file_path.to_path_buf(),
        message: "Missing 'Version' field in PKG-INFO".to_string(),
    })?;

    Ok(PythonMetadata {
        name,
        version,
        dependencies,
    })
}

/// Parse a Requires-Dist specification
/// Format: package-name (>=version,<version) ; extra == "extra_name"
/// Examples:
///   - "requests (>=2.0.0)"
///   - "urllib3 (<3,>=1.21.1)"
///   - "pytest ; extra == 'dev'"
fn parse_requires_dist(spec: &str) -> Option<(String, String)> {
    // Remove extras/markers (everything after semicolon)
    let spec = spec.split(';').next()?.trim();

    if spec.is_empty() {
        return None;
    }

    // Split on parentheses to separate name and version
    if let Some(paren_pos) = spec.find('(') {
        let name = spec[..paren_pos].trim().to_string();
        let version_part = spec[paren_pos + 1..].trim_end_matches(')').trim();
        Some((name, version_part.to_string()))
    } else {
        // No version constraint specified
        Some((spec.to_string(), "*".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_parse_metadata() {
        let content = r#"Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Summary: Python HTTP for Humans.
Requires-Dist: charset-normalizer (<4,>=2)
Requires-Dist: idna (<4,>=2.5)
Requires-Dist: urllib3 (<3,>=1.21.1)
"#;

        let metadata = parse_metadata(content, &PathBuf::from("METADATA")).unwrap();

        assert_eq!(metadata.name, "requests");
        assert_eq!(metadata.version, "2.31.0");
        assert_eq!(metadata.dependencies.len(), 3);
        assert_eq!(metadata.dependencies[0].0, "charset-normalizer");
        assert_eq!(metadata.dependencies[0].1, "<4,>=2");
        assert_eq!(metadata.dependencies[1].0, "idna");
        assert_eq!(metadata.dependencies[2].0, "urllib3");
    }

    #[test]
    fn test_parse_metadata_with_extras() {
        let content = r#"Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Requires-Dist: pytest ; extra == 'dev'
Requires-Dist: urllib3 (<3,>=1.21.1)
"#;

        let metadata = parse_metadata(content, &PathBuf::from("METADATA")).unwrap();

        assert_eq!(metadata.dependencies.len(), 2);
        // Extras should be ignored
        assert_eq!(metadata.dependencies[0].0, "pytest");
        assert_eq!(metadata.dependencies[1].0, "urllib3");
    }

    #[test]
    fn test_parse_pkg_info() {
        let content = r#"Metadata-Version: 1.1
Name: simplejson
Version: 3.19.1
Summary: Simple, fast, extensible JSON encoder/decoder
"#;

        let metadata = parse_pkg_info(content, &PathBuf::from("PKG-INFO")).unwrap();

        assert_eq!(metadata.name, "simplejson");
        assert_eq!(metadata.version, "3.19.1");
    }

    #[test]
    fn test_parse_pkg_info_with_requires() {
        let content = r#"Metadata-Version: 1.1
Name: oldpackage
Version: 1.0.0
Requires: requests
Requires: urllib3
"#;

        let metadata = parse_pkg_info(content, &PathBuf::from("PKG-INFO")).unwrap();

        assert_eq!(metadata.dependencies.len(), 2);
        assert_eq!(metadata.dependencies[0].0, "requests");
        assert_eq!(metadata.dependencies[0].1, "*");
        assert_eq!(metadata.dependencies[1].0, "urllib3");
    }

    #[test]
    fn test_parse_requires_dist_simple() {
        let result = parse_requires_dist("requests").unwrap();
        assert_eq!(result.0, "requests");
        assert_eq!(result.1, "*");
    }

    #[test]
    fn test_parse_requires_dist_with_version() {
        let result = parse_requires_dist("requests (>=2.0.0)").unwrap();
        assert_eq!(result.0, "requests");
        assert_eq!(result.1, ">=2.0.0");
    }

    #[test]
    fn test_parse_requires_dist_with_complex_version() {
        let result = parse_requires_dist("urllib3 (<3,>=1.21.1)").unwrap();
        assert_eq!(result.0, "urllib3");
        assert_eq!(result.1, "<3,>=1.21.1");
    }

    #[test]
    fn test_parse_requires_dist_with_extras() {
        let result = parse_requires_dist("pytest ; extra == 'dev'").unwrap();
        assert_eq!(result.0, "pytest");
        assert_eq!(result.1, "*");
    }

    #[test]
    fn test_parse_requires_dist_with_version_and_extras() {
        let result = parse_requires_dist("pytest (>=6.0) ; extra == 'dev'").unwrap();
        assert_eq!(result.0, "pytest");
        assert_eq!(result.1, ">=6.0");
    }

    #[test]
    fn test_parse_metadata_missing_name() {
        let content = "Version: 1.0.0\n";
        let result = parse_metadata(content, &PathBuf::from("METADATA"));
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_metadata_missing_version() {
        let content = "Name: test\n";
        let result = parse_metadata(content, &PathBuf::from("METADATA"));
        assert!(result.is_err());
    }
}
