//! Rust semantic versioning support
//!
//! This module provides version parsing and comparison for Rust packages.
//! Future: integrate semver crate for full Cargo compatibility.

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
