//! Python PEP 440 versioning support
//!
//! This module provides version parsing and comparison for Python packages.
//! Future: integrate pep440_rs crate for full PEP 440 compliance.

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
