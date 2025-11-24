//! Version matching and comparison utilities
//!
//! This module provides version comparison functionality across different ecosystems,
//! including exact matching and range satisfaction checking.

use crate::models::{Ecosystem, ScanError};
use crate::version::{node_semver, python_pep440, rust_semver};

/// Version matcher for comparing versions across ecosystems
pub struct VersionMatcher;

impl VersionMatcher {
    /// Create a new VersionMatcher
    pub fn new() -> Self {
        Self
    }

    /// Check if two versions are exactly equal
    pub fn exact_match(&self, v1: &str, v2: &str) -> bool {
        v1.trim() == v2.trim()
    }

    /// Check if a version satisfies a version range
    ///
    /// # Arguments
    ///
    /// * `version` - The exact version to check (e.g., "18.2.0")
    /// * `range` - The version range/constraint (e.g., "^18.0.0", ">=2.0.0")
    /// * `ecosystem` - The ecosystem to use for version parsing
    pub fn satisfies_range(
        &self,
        version: &str,
        range: &str,
        ecosystem: Ecosystem,
    ) -> Result<bool, ScanError> {
        match ecosystem {
            Ecosystem::Node => node_semver::satisfies(version, range),
            Ecosystem::Python => python_pep440::satisfies(version, range),
            Ecosystem::Rust => rust_semver::satisfies(version, range),
        }
    }

    /// Detect version mismatch between Has and Should classifications
    pub fn detect_version_mismatch(&self, has_version: &str, should_version: &str) -> bool {
        !self.exact_match(has_version, should_version)
    }

    /// Detect constraint violation (Should doesn't satisfy Can range)
    pub fn detect_constraint_violation(
        &self,
        should_version: &str,
        can_range: &str,
        ecosystem: Ecosystem,
    ) -> bool {
        match self.satisfies_range(should_version, can_range, ecosystem) {
            Ok(satisfies) => !satisfies,
            Err(_) => {
                // If we can't parse, assume no violation (be conservative)
                false
            }
        }
    }
}

impl Default for VersionMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        let matcher = VersionMatcher::new();
        assert!(matcher.exact_match("18.2.0", "18.2.0"));
        assert!(matcher.exact_match(" 18.2.0 ", "18.2.0"));
        assert!(!matcher.exact_match("18.2.0", "18.2.1"));
        assert!(!matcher.exact_match("18.2.0", "18.3.0"));
    }

    #[test]
    fn test_detect_version_mismatch() {
        let matcher = VersionMatcher::new();
        assert!(!matcher.detect_version_mismatch("18.2.0", "18.2.0"));
        assert!(matcher.detect_version_mismatch("18.2.0", "18.2.1"));
        assert!(matcher.detect_version_mismatch("18.2.0", "17.0.0"));
    }

    #[test]
    fn test_satisfies_range_node() {
        let matcher = VersionMatcher::new();

        // Caret ranges
        assert!(matcher
            .satisfies_range("18.2.0", "^18.0.0", Ecosystem::Node)
            .unwrap());
        assert!(!matcher
            .satisfies_range("17.0.0", "^18.0.0", Ecosystem::Node)
            .unwrap());

        // Tilde ranges
        assert!(matcher
            .satisfies_range("1.2.3", "~1.2.0", Ecosystem::Node)
            .unwrap());
        assert!(!matcher
            .satisfies_range("1.3.0", "~1.2.0", Ecosystem::Node)
            .unwrap());

        // Exact version
        assert!(matcher
            .satisfies_range("1.2.3", "1.2.3", Ecosystem::Node)
            .unwrap());
        assert!(!matcher
            .satisfies_range("1.2.4", "1.2.3", Ecosystem::Node)
            .unwrap());
    }

    #[test]
    fn test_satisfies_range_python() {
        let matcher = VersionMatcher::new();

        // Greater than or equal
        assert!(matcher
            .satisfies_range("2.31.0", ">=2.0.0", Ecosystem::Python)
            .unwrap());
        assert!(!matcher
            .satisfies_range("1.9.0", ">=2.0.0", Ecosystem::Python)
            .unwrap());

        // Compatible release
        assert!(matcher
            .satisfies_range("2.31.0", "~=2.30", Ecosystem::Python)
            .unwrap());
    }

    #[test]
    fn test_satisfies_range_rust() {
        let matcher = VersionMatcher::new();

        // Caret requirements
        assert!(matcher
            .satisfies_range("1.2.3", "^1.2.0", Ecosystem::Rust)
            .unwrap());
        assert!(!matcher
            .satisfies_range("2.0.0", "^1.2.0", Ecosystem::Rust)
            .unwrap());
    }

    #[test]
    fn test_detect_constraint_violation() {
        let matcher = VersionMatcher::new();

        // No violation - version satisfies range
        assert!(!matcher.detect_constraint_violation("18.2.0", "^18.0.0", Ecosystem::Node));

        // Violation - version doesn't satisfy range
        assert!(matcher.detect_constraint_violation("17.0.0", "^18.0.0", Ecosystem::Node));
    }
}
