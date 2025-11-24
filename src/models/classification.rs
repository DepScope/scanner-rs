//! Classification system for dependency analysis (HAS/SHOULD/CAN)
//!
//! This module provides a three-tier classification system for analyzing dependencies
//! across their lifecycle from declaration to installation:
//!
//! - **HAS**: Package is physically installed in the filesystem (node_modules, site-packages)
//! - **SHOULD**: Package version is specified in a lock file (the intended installation)
//! - **CAN**: Package is declared in a manifest with a version range (allowed versions)
//!
//! This classification system enables supply chain security analysis by identifying
//! which systems have vulnerable packages actually installed versus merely declared.
//!
//! # Example
//!
//! ```rust
//! use scanner::models::{Classification, ClassifiedDependency, Ecosystem};
//! use std::path::PathBuf;
//!
//! let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
//!
//! // Add HAS classification (installed)
//! dep.add_classification(
//!     Classification::Has,
//!     "18.2.0".to_string(),
//!     PathBuf::from("/app/node_modules/react"),
//! );
//!
//! // Add SHOULD classification (locked)
//! dep.add_classification(
//!     Classification::Should,
//!     "18.2.0".to_string(),
//!     PathBuf::from("/app/package-lock.json"),
//! );
//!
//! // Add CAN classification (declared)
//! dep.add_classification(
//!     Classification::Can,
//!     "^18.0.0".to_string(),
//!     PathBuf::from("/app/package.json"),
//! );
//!
//! assert_eq!(dep.get_classifications().len(), 3);
//! assert_eq!(dep.primary_classification(), Some(Classification::Has));
//! ```

use super::dependency::Ecosystem;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Classification of a dependency based on its source
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Classification {
    /// Package is physically installed (found in node_modules, site-packages, etc.)
    Has,
    /// Package version is specified in a lock file (intended installation)
    Should,
    /// Package is declared in a manifest with a version range (allowed versions)
    Can,
}

impl std::fmt::Display for Classification {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Classification::Has => write!(f, "HAS"),
            Classification::Should => write!(f, "SHOULD"),
            Classification::Can => write!(f, "CAN"),
        }
    }
}

/// A dependency with multiple classifications and associated metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedDependency {
    /// Package name
    pub name: String,

    /// Classifications with their associated versions
    /// - Has: exact installed version
    /// - Should: exact locked version
    /// - Can: version range from manifest
    pub classifications: HashMap<Classification, String>,

    /// Ecosystem (Node, Python, Rust)
    pub ecosystem: Ecosystem,

    /// Application root directory (nearest manifest file)
    pub application_root: Option<PathBuf>,

    /// Application name (extracted from manifest)
    pub application_name: Option<String>,

    /// Installed package path (for Has classification)
    pub installed_path: Option<PathBuf>,

    /// Source files for each classification
    pub source_files: HashMap<Classification, PathBuf>,

    /// Version mismatch between Has and Should
    pub has_version_mismatch: bool,

    /// Constraint violation (Should doesn't satisfy Can range)
    pub has_constraint_violation: bool,

    /// Parent package name (for dependency tree)
    pub parent_package: Option<String>,

    /// Direct dependencies of this package
    pub dependencies: Vec<String>,

    /// Security status (for infected package detection)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub security: Option<String>,
}

impl ClassifiedDependency {
    /// Create a new ClassifiedDependency with the given name and ecosystem
    pub fn new(name: String, ecosystem: Ecosystem) -> Self {
        Self {
            name,
            classifications: HashMap::new(),
            ecosystem,
            application_root: None,
            application_name: None,
            installed_path: None,
            source_files: HashMap::new(),
            has_version_mismatch: false,
            has_constraint_violation: false,
            parent_package: None,
            dependencies: Vec::new(),
            security: None,
        }
    }

    /// Add a classification with version and source file
    pub fn add_classification(
        &mut self,
        classification: Classification,
        version: String,
        source_file: PathBuf,
    ) {
        self.classifications.insert(classification, version);
        self.source_files.insert(classification, source_file);
    }

    /// Get the version for a specific classification
    pub fn get_version(&self, classification: Classification) -> Option<&str> {
        self.classifications
            .get(&classification)
            .map(|s| s.as_str())
    }

    /// Get the source file for a specific classification
    pub fn get_source_file(&self, classification: Classification) -> Option<&PathBuf> {
        self.source_files.get(&classification)
    }

    /// Check if this dependency has a specific classification
    pub fn has_classification(&self, classification: Classification) -> bool {
        self.classifications.contains_key(&classification)
    }

    /// Get all classifications for this dependency
    pub fn get_classifications(&self) -> Vec<Classification> {
        let mut classifications: Vec<_> = self.classifications.keys().copied().collect();
        // Sort by priority: Has, Should, Can
        classifications.sort_by_key(|c| match c {
            Classification::Has => 0,
            Classification::Should => 1,
            Classification::Can => 2,
        });
        classifications
    }

    /// Get the highest priority classification
    pub fn primary_classification(&self) -> Option<Classification> {
        self.get_classifications().first().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classification_display() {
        assert_eq!(Classification::Has.to_string(), "HAS");
        assert_eq!(Classification::Should.to_string(), "SHOULD");
        assert_eq!(Classification::Can.to_string(), "CAN");
    }

    #[test]
    fn test_new_classified_dependency() {
        let dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        assert_eq!(dep.name, "react");
        assert_eq!(dep.ecosystem, Ecosystem::Node);
        assert!(dep.classifications.is_empty());
        assert!(!dep.has_version_mismatch);
        assert!(!dep.has_constraint_violation);
    }

    #[test]
    fn test_add_classification() {
        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        assert!(dep.has_classification(Classification::Has));
        assert_eq!(dep.get_version(Classification::Has), Some("18.2.0"));
        assert_eq!(
            dep.get_source_file(Classification::Has),
            Some(&PathBuf::from("/app/node_modules/react"))
        );
    }

    #[test]
    fn test_get_classifications_sorted() {
        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Can,
            "^18.0.0".to_string(),
            PathBuf::from("/app/package.json"),
        );
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );
        dep.add_classification(
            Classification::Should,
            "18.2.0".to_string(),
            PathBuf::from("/app/package-lock.json"),
        );

        let classifications = dep.get_classifications();
        assert_eq!(classifications.len(), 3);
        assert_eq!(classifications[0], Classification::Has);
        assert_eq!(classifications[1], Classification::Should);
        assert_eq!(classifications[2], Classification::Can);
    }

    #[test]
    fn test_primary_classification() {
        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        assert_eq!(dep.primary_classification(), None);

        dep.add_classification(
            Classification::Should,
            "18.2.0".to_string(),
            PathBuf::from("/app/package-lock.json"),
        );
        assert_eq!(dep.primary_classification(), Some(Classification::Should));

        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );
        assert_eq!(dep.primary_classification(), Some(Classification::Has));
    }
}
