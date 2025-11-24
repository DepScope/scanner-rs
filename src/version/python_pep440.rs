//! Python PEP 440 versioning support
//!
//! This module provides version parsing and comparison for Python packages.
//! Future: integrate pep440_rs crate for full PEP 440 compliance.

use crate::models::ScanError;

/// Python version wrapper
pub struct PythonVersion {
    raw: String,
}

impl PythonVersion {
    /// Parse a Python version string
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

/// Check if a version satisfies a PEP 440 specifier
///
/// This is a simplified implementation. For production use, integrate pep440_rs crate.
pub fn satisfies(version: &str, specifier: &str) -> Result<bool, ScanError> {
    let version = version.trim();
    let specifier = specifier.trim();

    // Exact match
    if version == specifier {
        return Ok(true);
    }

    // Parse version
    let version_parts = parse_version_parts(version)?;

    // Handle >= specifier
    if specifier.starts_with(">=") {
        let spec_version = &specifier[2..].trim();
        let spec_parts = parse_version_parts(spec_version)?;
        return Ok(version_parts >= spec_parts);
    }

    // Handle > specifier
    if specifier.starts_with('>') {
        let spec_version = &specifier[1..].trim();
        let spec_parts = parse_version_parts(spec_version)?;
        return Ok(version_parts > spec_parts);
    }

    // Handle <= specifier
    if specifier.starts_with("<=") {
        let spec_version = &specifier[2..].trim();
        let spec_parts = parse_version_parts(spec_version)?;
        return Ok(version_parts <= spec_parts);
    }

    // Handle < specifier
    if specifier.starts_with('<') {
        let spec_version = &specifier[1..].trim();
        let spec_parts = parse_version_parts(spec_version)?;
        return Ok(version_parts < spec_parts);
    }

    // Handle == specifier
    if specifier.starts_with("==") {
        let spec_version = specifier[2..].trim();
        return Ok(version == spec_version);
    }

    // Handle ~= compatible release (e.g., ~=2.2 matches >=2.2, <3.0)
    if specifier.starts_with("~=") {
        let spec_version = &specifier[2..].trim();
        let spec_parts = parse_version_parts(spec_version)?;
        return Ok(version_parts.0 == spec_parts.0
            && (version_parts.1 > spec_parts.1
                || (version_parts.1 == spec_parts.1 && version_parts.2 >= spec_parts.2)));
    }

    // Default: exact match
    Ok(version == specifier)
}

fn parse_version_parts(version: &str) -> Result<(u32, u32, u32), ScanError> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.is_empty() {
        return Err(ScanError::Parse {
            file: std::path::PathBuf::from("version"),
            message: format!("Invalid version format: {}", version),
        });
    }

    let major = parts[0].parse::<u32>().map_err(|_| ScanError::Parse {
        file: std::path::PathBuf::from("version"),
        message: format!("Invalid major version: {}", parts[0]),
    })?;

    let minor = if parts.len() > 1 {
        parts[1].parse::<u32>().map_err(|_| ScanError::Parse {
            file: std::path::PathBuf::from("version"),
            message: format!("Invalid minor version: {}", parts[1]),
        })?
    } else {
        0
    };

    let patch = if parts.len() > 2 {
        parts[2]
            .split(|c: char| !c.is_numeric())
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0)
    } else {
        0
    };

    Ok((major, minor, patch))
}
