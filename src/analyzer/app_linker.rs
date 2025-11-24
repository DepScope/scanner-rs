//! Application linker for associating dependencies with their declaring applications
//!
//! This module finds the nearest manifest file (application root) for each
//! installed package and links them together.

use crate::models::{Application, ClassifiedDependency, Ecosystem};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Application linker for finding and linking application roots
pub struct ApplicationLinker;

impl ApplicationLinker {
    /// Create a new ApplicationLinker
    pub fn new() -> Self {
        Self
    }

    /// Link classified dependencies to their application roots
    ///
    /// For each dependency with an installed path, searches parent directories
    /// for manifest files and groups dependencies by application.
    pub fn link_to_applications(
        &self,
        mut dependencies: Vec<ClassifiedDependency>,
    ) -> Vec<Application> {
        // Cache for manifest file locations
        let mut manifest_cache: HashMap<PathBuf, Option<(PathBuf, String, Ecosystem)>> =
            HashMap::new();

        // Update dependencies with application information
        for dep in &mut dependencies {
            if let Some(installed_path) = &dep.installed_path {
                if let Some((root_path, app_name, _ecosystem)) =
                    self.find_application_root(installed_path, &mut manifest_cache)
                {
                    dep.application_root = Some(root_path);
                    dep.application_name = Some(app_name);
                }
            }
        }

        // Group dependencies by application root
        let mut apps: HashMap<PathBuf, Application> = HashMap::new();

        for dep in dependencies {
            if let Some(root_path) = &dep.application_root {
                let app = apps.entry(root_path.clone()).or_insert_with(|| {
                    let app_name = dep
                        .application_name
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());
                    let manifest_path = self.find_manifest_file(root_path, dep.ecosystem);
                    Application::new(
                        app_name,
                        root_path.clone(),
                        manifest_path.unwrap_or_else(|| root_path.clone()),
                        dep.ecosystem,
                    )
                });
                app.add_dependency(dep);
            }
        }

        apps.into_values().collect()
    }

    /// Find the application root for an installed package
    ///
    /// Traverses parent directories looking for manifest files.
    fn find_application_root(
        &self,
        installed_path: &Path,
        cache: &mut HashMap<PathBuf, Option<(PathBuf, String, Ecosystem)>>,
    ) -> Option<(PathBuf, String, Ecosystem)> {
        let mut current = installed_path.to_path_buf();

        // Traverse up to find manifest file
        loop {
            current = current.parent()?.to_path_buf();

            // Check cache
            if let Some(cached) = cache.get(&current) {
                return cached.clone();
            }

            // Check for Node.js manifest
            if let Some((name, ecosystem)) = self.check_node_manifest(&current) {
                let result = Some((current.clone(), name, ecosystem));
                cache.insert(current.clone(), result.clone());
                return result;
            }

            // Check for Python manifest
            if let Some((name, ecosystem)) = self.check_python_manifest(&current) {
                let result = Some((current.clone(), name, ecosystem));
                cache.insert(current.clone(), result.clone());
                return result;
            }

            // Check for Rust manifest
            if let Some((name, ecosystem)) = self.check_rust_manifest(&current) {
                let result = Some((current.clone(), name, ecosystem));
                cache.insert(current.clone(), result.clone());
                return result;
            }

            // Stop at filesystem root
            if current.parent().is_none() {
                cache.insert(current, None);
                return None;
            }
        }
    }

    /// Check for Node.js manifest (package.json)
    fn check_node_manifest(&self, dir: &Path) -> Option<(String, Ecosystem)> {
        let package_json = dir.join("package.json");
        if package_json.exists() {
            if let Ok(content) = fs::read_to_string(&package_json) {
                if let Ok(json) = serde_json::from_str::<Value>(&content) {
                    if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                        return Some((name.to_string(), Ecosystem::Node));
                    }
                }
            }
        }
        None
    }

    /// Check for Python manifest (pyproject.toml)
    fn check_python_manifest(&self, dir: &Path) -> Option<(String, Ecosystem)> {
        let pyproject = dir.join("pyproject.toml");
        if pyproject.exists() {
            if let Ok(content) = fs::read_to_string(&pyproject) {
                // Simple TOML parsing - look for [project] name or [tool.poetry] name
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("name") && line.contains('=') {
                        if let Some(name_part) = line.split('=').nth(1) {
                            let name = name_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            if !name.is_empty() {
                                return Some((name, Ecosystem::Python));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Check for Rust manifest (Cargo.toml)
    fn check_rust_manifest(&self, dir: &Path) -> Option<(String, Ecosystem)> {
        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                // Simple TOML parsing - look for [package] name
                let mut in_package_section = false;
                for line in content.lines() {
                    let line = line.trim();
                    if line == "[package]" {
                        in_package_section = true;
                        continue;
                    }
                    if line.starts_with('[') {
                        in_package_section = false;
                    }
                    if in_package_section && line.starts_with("name") && line.contains('=') {
                        if let Some(name_part) = line.split('=').nth(1) {
                            let name = name_part
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string();
                            if !name.is_empty() {
                                return Some((name, Ecosystem::Rust));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the manifest file path for a given root directory
    fn find_manifest_file(&self, root: &Path, ecosystem: Ecosystem) -> Option<PathBuf> {
        match ecosystem {
            Ecosystem::Node => {
                let path = root.join("package.json");
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            }
            Ecosystem::Python => {
                let path = root.join("pyproject.toml");
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            }
            Ecosystem::Rust => {
                let path = root.join("Cargo.toml");
                if path.exists() {
                    Some(path)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for ApplicationLinker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Classification;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_find_node_application_root() {
        let temp_dir = TempDir::new().unwrap();
        let app_root = temp_dir.path().join("myapp");
        fs::create_dir_all(&app_root).unwrap();

        // Create package.json
        fs::write(
            app_root.join("package.json"),
            r#"{"name": "myapp", "version": "1.0.0"}"#,
        )
        .unwrap();

        // Create node_modules
        let node_modules = app_root.join("node_modules");
        let react_dir = node_modules.join("react");
        fs::create_dir_all(&react_dir).unwrap();

        let linker = ApplicationLinker::new();
        let mut cache = HashMap::new();
        let result = linker.find_application_root(&react_dir, &mut cache);

        assert!(result.is_some());
        let (root, name, ecosystem) = result.unwrap();
        assert_eq!(root, app_root);
        assert_eq!(name, "myapp");
        assert_eq!(ecosystem, Ecosystem::Node);
    }

    #[test]
    fn test_find_python_application_root() {
        let temp_dir = TempDir::new().unwrap();
        let app_root = temp_dir.path().join("myapp");
        fs::create_dir_all(&app_root).unwrap();

        // Create pyproject.toml
        fs::write(
            app_root.join("pyproject.toml"),
            r#"[project]
name = "myapp"
version = "1.0.0"
"#,
        )
        .unwrap();

        // Create site-packages
        let site_packages = app_root.join(".venv/lib/python3.11/site-packages");
        let requests_dir = site_packages.join("requests");
        fs::create_dir_all(&requests_dir).unwrap();

        let linker = ApplicationLinker::new();
        let mut cache = HashMap::new();
        let result = linker.find_application_root(&requests_dir, &mut cache);

        assert!(result.is_some());
        let (root, name, ecosystem) = result.unwrap();
        assert_eq!(root, app_root);
        assert_eq!(name, "myapp");
        assert_eq!(ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_link_to_applications() {
        let temp_dir = TempDir::new().unwrap();
        let app_root = temp_dir.path().join("myapp");
        fs::create_dir_all(&app_root).unwrap();

        fs::write(
            app_root.join("package.json"),
            r#"{"name": "myapp", "version": "1.0.0"}"#,
        )
        .unwrap();

        let node_modules = app_root.join("node_modules");
        fs::create_dir_all(&node_modules).unwrap();

        let linker = ApplicationLinker::new();

        let mut dep1 = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep1.installed_path = Some(node_modules.join("react"));
        dep1.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            node_modules.join("react"),
        );

        let mut dep2 = ClassifiedDependency::new("lodash".to_string(), Ecosystem::Node);
        dep2.installed_path = Some(node_modules.join("lodash"));
        dep2.add_classification(
            Classification::Has,
            "4.17.21".to_string(),
            node_modules.join("lodash"),
        );

        let apps = linker.link_to_applications(vec![dep1, dep2]);

        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "myapp");
        assert_eq!(apps[0].ecosystem, Ecosystem::Node);
        assert_eq!(apps[0].dependency_count(), 2);
        assert!(apps[0].has_dependency("react"));
        assert!(apps[0].has_dependency("lodash"));
    }

    #[test]
    fn test_link_multiple_applications() {
        let temp_dir = TempDir::new().unwrap();

        // Create first app
        let app1_root = temp_dir.path().join("app1");
        fs::create_dir_all(&app1_root).unwrap();
        fs::write(
            app1_root.join("package.json"),
            r#"{"name": "app1", "version": "1.0.0"}"#,
        )
        .unwrap();
        let nm1 = app1_root.join("node_modules");
        fs::create_dir_all(&nm1).unwrap();

        // Create second app
        let app2_root = temp_dir.path().join("app2");
        fs::create_dir_all(&app2_root).unwrap();
        fs::write(
            app2_root.join("package.json"),
            r#"{"name": "app2", "version": "1.0.0"}"#,
        )
        .unwrap();
        let nm2 = app2_root.join("node_modules");
        fs::create_dir_all(&nm2).unwrap();

        let linker = ApplicationLinker::new();

        let mut dep1 = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep1.installed_path = Some(nm1.join("react"));

        let mut dep2 = ClassifiedDependency::new("lodash".to_string(), Ecosystem::Node);
        dep2.installed_path = Some(nm2.join("lodash"));

        let apps = linker.link_to_applications(vec![dep1, dep2]);

        assert_eq!(apps.len(), 2);
        assert!(apps.iter().any(|a| a.name == "app1"));
        assert!(apps.iter().any(|a| a.name == "app2"));
    }

    #[test]
    fn test_no_application_root() {
        let temp_dir = TempDir::new().unwrap();
        let orphan_dir = temp_dir.path().join("orphan/node_modules/react");
        fs::create_dir_all(&orphan_dir).unwrap();

        let linker = ApplicationLinker::new();

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.installed_path = Some(orphan_dir);

        let apps = linker.link_to_applications(vec![dep]);

        // Should return empty since no application root was found
        assert_eq!(apps.len(), 0);
    }
}
