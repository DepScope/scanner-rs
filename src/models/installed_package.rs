//! Installed package data structures

use super::dependency::Ecosystem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A dependency specification (name and version constraint)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencySpec {
    /// Dependency package name
    pub name: String,

    /// Version constraint (e.g., "^1.0.0", ">=2.0.0", "*")
    pub version_constraint: String,
}

impl DependencySpec {
    /// Create a new DependencySpec
    pub fn new(name: String, version_constraint: String) -> Self {
        Self {
            name,
            version_constraint,
        }
    }
}

/// An installed package found in the filesystem
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledPackage {
    /// Package name
    pub name: String,

    /// Installed version
    pub version: String,

    /// Installation path (e.g., /app/node_modules/react)
    pub path: PathBuf,

    /// Ecosystem (Node, Python, Rust)
    pub ecosystem: Ecosystem,

    /// Direct dependencies declared by this package
    pub dependencies: Vec<DependencySpec>,
}

impl InstalledPackage {
    /// Create a new InstalledPackage
    pub fn new(name: String, version: String, path: PathBuf, ecosystem: Ecosystem) -> Self {
        Self {
            name,
            version,
            path,
            ecosystem,
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to this package
    pub fn add_dependency(&mut self, name: String, version_constraint: String) {
        self.dependencies
            .push(DependencySpec::new(name, version_constraint));
    }

    /// Get all dependencies
    pub fn get_dependencies(&self) -> &[DependencySpec] {
        &self.dependencies
    }

    /// Check if this package has a specific dependency
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }

    /// Find a dependency by name
    pub fn find_dependency(&self, name: &str) -> Option<&DependencySpec> {
        self.dependencies.iter().find(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_spec_creation() {
        let spec = DependencySpec::new("react".to_string(), "^18.0.0".to_string());
        assert_eq!(spec.name, "react");
        assert_eq!(spec.version_constraint, "^18.0.0");
    }

    #[test]
    fn test_installed_package_creation() {
        let pkg = InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        );

        assert_eq!(pkg.name, "react");
        assert_eq!(pkg.version, "18.2.0");
        assert_eq!(pkg.path, PathBuf::from("/app/node_modules/react"));
        assert_eq!(pkg.ecosystem, Ecosystem::Node);
        assert_eq!(pkg.dependencies.len(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let mut pkg = InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        );

        pkg.add_dependency("loose-envify".to_string(), "^1.1.0".to_string());
        pkg.add_dependency("scheduler".to_string(), "^0.23.0".to_string());

        assert_eq!(pkg.dependencies.len(), 2);
        assert!(pkg.has_dependency("loose-envify"));
        assert!(pkg.has_dependency("scheduler"));
        assert!(!pkg.has_dependency("nonexistent"));
    }

    #[test]
    fn test_find_dependency() {
        let mut pkg = InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        );

        pkg.add_dependency("loose-envify".to_string(), "^1.1.0".to_string());

        let dep = pkg.find_dependency("loose-envify");
        assert!(dep.is_some());
        assert_eq!(dep.unwrap().version_constraint, "^1.1.0");

        let not_found = pkg.find_dependency("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_get_dependencies() {
        let mut pkg = InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        );

        pkg.add_dependency("loose-envify".to_string(), "^1.1.0".to_string());
        pkg.add_dependency("scheduler".to_string(), "^0.23.0".to_string());

        let deps = pkg.get_dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "loose-envify");
        assert_eq!(deps[1].name, "scheduler");
    }
}
