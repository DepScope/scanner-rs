//! Application root tracking for linking dependencies to their declaring applications

use super::classification::ClassifiedDependency;
use super::dependency::Ecosystem;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// An application root representing a project with dependencies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Application {
    /// Application name (from package.json, pyproject.toml, Cargo.toml, etc.)
    pub name: String,

    /// Absolute path to application root directory
    pub root_path: PathBuf,

    /// Path to the manifest file
    pub manifest_path: PathBuf,

    /// Ecosystem (Node, Python, Rust)
    pub ecosystem: Ecosystem,

    /// All dependencies associated with this application
    pub dependencies: Vec<ClassifiedDependency>,
}

impl Application {
    /// Create a new Application
    pub fn new(
        name: String,
        root_path: PathBuf,
        manifest_path: PathBuf,
        ecosystem: Ecosystem,
    ) -> Self {
        Self {
            name,
            root_path,
            manifest_path,
            ecosystem,
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to this application
    pub fn add_dependency(&mut self, dependency: ClassifiedDependency) {
        self.dependencies.push(dependency);
    }

    /// Get all dependencies for this application
    pub fn get_dependencies(&self) -> &[ClassifiedDependency] {
        &self.dependencies
    }

    /// Find a dependency by name
    pub fn find_dependency(&self, name: &str) -> Option<&ClassifiedDependency> {
        self.dependencies.iter().find(|d| d.name == name)
    }

    /// Find a mutable dependency by name
    pub fn find_dependency_mut(&mut self, name: &str) -> Option<&mut ClassifiedDependency> {
        self.dependencies.iter_mut().find(|d| d.name == name)
    }

    /// Get the number of dependencies
    pub fn dependency_count(&self) -> usize {
        self.dependencies.len()
    }

    /// Check if this application has a specific dependency
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.iter().any(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::classification::Classification;

    #[test]
    fn test_new_application() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        assert_eq!(app.name, "myapp");
        assert_eq!(app.root_path, PathBuf::from("/home/user/myapp"));
        assert_eq!(
            app.manifest_path,
            PathBuf::from("/home/user/myapp/package.json")
        );
        assert_eq!(app.ecosystem, Ecosystem::Node);
        assert_eq!(app.dependency_count(), 0);
    }

    #[test]
    fn test_add_dependency() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/home/user/myapp/node_modules/react"),
        );

        app.add_dependency(dep);
        assert_eq!(app.dependency_count(), 1);
        assert!(app.has_dependency("react"));
    }

    #[test]
    fn test_find_dependency() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/home/user/myapp/node_modules/react"),
        );

        app.add_dependency(dep);

        let found = app.find_dependency("react");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "react");

        let not_found = app.find_dependency("lodash");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_find_dependency_mut() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        let dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        app.add_dependency(dep);

        if let Some(dep) = app.find_dependency_mut("react") {
            dep.add_classification(
                Classification::Should,
                "18.2.0".to_string(),
                PathBuf::from("/home/user/myapp/package-lock.json"),
            );
        }

        let dep = app.find_dependency("react").unwrap();
        assert!(dep.has_classification(Classification::Should));
    }

    #[test]
    fn test_get_dependencies() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/home/user/myapp"),
            PathBuf::from("/home/user/myapp/package.json"),
            Ecosystem::Node,
        );

        app.add_dependency(ClassifiedDependency::new(
            "react".to_string(),
            Ecosystem::Node,
        ));
        app.add_dependency(ClassifiedDependency::new(
            "lodash".to_string(),
            Ecosystem::Node,
        ));

        let deps = app.get_dependencies();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].name, "react");
        assert_eq!(deps[1].name, "lodash");
    }
}
