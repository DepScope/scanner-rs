//! Node.js semantic versioning support
//!
//! This module provides version parsing and comparison for Node.js packages.
//! Future: integrate node-semver crate for full npm compatibility.

use crate::models::ScanError;

/// Node.js version wrapper
pub struct NodeVersion {
    raw: String,
}

impl NodeVersion {
    /// Parse a Node.js version string
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

/// Check if a version satisfies a range
///
/// This is a simplified implementation. For production use, integrate node-semver crate.
pub fn satisfies(version: &str, range: &str) -> Result<bool, ScanError> {
    // Simplified version matching
    let version = version.trim();
    let range = range.trim();

    // Exact match
    if version == range {
        return Ok(true);
    }

    // Parse version components
    let version_parts = parse_version_parts(version)?;

    // Handle caret ranges (^1.2.3 allows >=1.2.3 <2.0.0)
    if let Some(range_version) = range.strip_prefix('^') {
        let range_parts = parse_version_parts(range_version)?;
        return Ok(version_parts.0 == range_parts.0
            && (version_parts.1 > range_parts.1
                || (version_parts.1 == range_parts.1 && version_parts.2 >= range_parts.2)));
    }

    // Handle tilde ranges (~1.2.3 allows >=1.2.3 <1.3.0)
    if let Some(range_version) = range.strip_prefix('~') {
        let range_parts = parse_version_parts(range_version)?;
        return Ok(version_parts.0 == range_parts.0
            && version_parts.1 == range_parts.1
            && version_parts.2 >= range_parts.2);
    }

    // Handle >= ranges
    if let Some(stripped) = range.strip_prefix(">=") {
        let range_version = &stripped.trim();
        let range_parts = parse_version_parts(range_version)?;
        return Ok(version_parts >= range_parts);
    }

    // Handle > ranges
    if let Some(stripped) = range.strip_prefix('>') {
        let range_version = &stripped.trim();
        let range_parts = parse_version_parts(range_version)?;
        return Ok(version_parts > range_parts);
    }

    // Handle wildcard (*)
    if range == "*" || range == "x" || range == "X" {
        return Ok(true);
    }

    // Default: exact match
    Ok(version == range)
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
