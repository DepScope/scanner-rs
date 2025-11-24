//! Node.js semantic versioning support
//!
//! This module provides version parsing and comparison for Node.js packages.
//! Future: integrate node-semver crate for full npm compatibility.

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
