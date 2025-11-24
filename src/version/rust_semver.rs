//! Rust semantic versioning support
//!
//! This module provides version parsing and comparison for Rust packages.
//! Future: integrate semver crate for full Cargo compatibility.

use crate::models::ScanError;

/// Rust version wrapper
pub struct RustVersion {
    raw: String,
}

impl RustVersion {
    /// Parse a Rust version string
    pub fn parse(version: &str) -> Result<Self, String> {
        Ok(Self {
            raw: version.to_string(),
        })
    }

    /// Get the raw version string
    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

/// Check if a version satisfies a Cargo version requirement
///
/// This is a simplified implementation. For production use, integrate semver crate.
pub fn satisfies(version: &str, requirement: &str) -> Result<bool, ScanError> {
    let version = version.trim();
    let requirement = requirement.trim();

    // Exact match
    if version == requirement {
        return Ok(true);
    }

    // Parse version
    let version_parts = parse_version_parts(version)?;

    // Handle caret requirements (^1.2.3 is default in Cargo)
    if let Some(req_version) = requirement.strip_prefix('^') {
        let req_parts = parse_version_parts(req_version)?;
        return Ok(version_parts.0 == req_parts.0
            && (version_parts.1 > req_parts.1
                || (version_parts.1 == req_parts.1 && version_parts.2 >= req_parts.2)));
    }

    // Handle tilde requirements (~1.2.3)
    if let Some(req_version) = requirement.strip_prefix('~') {
        let req_parts = parse_version_parts(req_version)?;
        return Ok(version_parts.0 == req_parts.0
            && version_parts.1 == req_parts.1
            && version_parts.2 >= req_parts.2);
    }

    // Handle >= requirements
    if requirement.starts_with(">=") {
        let req_version = &requirement[2..].trim();
        let req_parts = parse_version_parts(req_version)?;
        return Ok(version_parts >= req_parts);
    }

    // Handle > requirements
    if requirement.starts_with('>') {
        let req_version = &requirement[1..].trim();
        let req_parts = parse_version_parts(req_version)?;
        return Ok(version_parts > req_parts);
    }

    // Handle wildcard (*)
    if requirement == "*" {
        return Ok(true);
    }

    // Default: treat as caret requirement (Cargo default)
    let req_parts = parse_version_parts(requirement)?;
    Ok(version_parts.0 == req_parts.0
        && (version_parts.1 > req_parts.1
            || (version_parts.1 == req_parts.1 && version_parts.2 >= req_parts.2)))
}

fn parse_version_parts(version: &str) -> Result<(u32, u32, u32), ScanError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() < 3 {
        return Err(ScanError::Parse {
            file: std::path::PathBuf::from("version"),
            message: format!("Invalid version format: {}", version),
        });
    }

    let major = parts[0].parse::<u32>().map_err(|_| ScanError::Parse {
        file: std::path::PathBuf::from("version"),
        message: format!("Invalid major version: {}", parts[0]),
    })?;

    let minor = parts[1].parse::<u32>().map_err(|_| ScanError::Parse {
        file: std::path::PathBuf::from("version"),
        message: format!("Invalid minor version: {}", parts[1]),
    })?;

    let patch = parts[2]
        .split('-')
        .next()
        .unwrap_or(parts[2])
        .parse::<u32>()
        .map_err(|_| ScanError::Parse {
            file: std::path::PathBuf::from("version"),
            message: format!("Invalid patch version: {}", parts[2]),
        })?;

    Ok((major, minor, patch))
}
