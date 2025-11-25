//! Classifier component for assigning HAS/SHOULD/CAN classifications
//!
//! This module creates separate ClassifiedDependency entries for each finding
//! without deduplication. Each installed package or declared dependency gets
//! its own entry, allowing for complete visibility of all findings.
//!
//! Classifications are assigned based on the source:
//!
//! - **HAS**: From installed package parsers (node_modules, site-packages)
//! - **SHOULD**: From lockfile parsers (package-lock.json, poetry.lock, etc.)
//! - **CAN**: From manifest parsers (package.json, pyproject.toml, etc.)

use crate::models::{
    Classification, ClassifiedDependency, DependencyRecord, FileType, InstalledPackage,
};

/// Classifier for assigning HAS/SHOULD/CAN classifications
pub struct Classifier;

impl Classifier {
    /// Create a new Classifier
    pub fn new() -> Self {
        Self
    }

    /// Classify dependency records and installed packages
    ///
    /// Creates separate entries for each unique finding without deduplication.
    /// Each installed package or declared dependency gets its own ClassifiedDependency entry.
    pub fn classify(
        &self,
        records: Vec<DependencyRecord>,
        installed: Vec<InstalledPackage>,
    ) -> Vec<ClassifiedDependency> {
        let mut results = Vec::new();

        // Process installed packages (HAS classification)
        // Each installed package gets its own entry
        for pkg in installed {
            let mut dep = ClassifiedDependency::new(pkg.name.clone(), pkg.ecosystem);
            dep.add_classification(Classification::Has, pkg.version.clone(), pkg.path.clone());
            dep.installed_path = Some(pkg.path.clone());

            // Set package_name_path from the installed path
            dep.package_name_path = Some(pkg.path.to_string_lossy().to_string());

            // Store dependencies for tree building
            for dep_spec in &pkg.dependencies {
                dep.dependencies.push(dep_spec.name.clone());
            }

            results.push(dep);
        }

        // Process dependency records (SHOULD and CAN classifications)
        // Each record gets its own entry
        for record in records {
            let mut dep = ClassifiedDependency::new(record.name.clone(), record.ecosystem);

            // Set package_name_path from the source file
            dep.package_name_path = Some(record.source_file.to_string_lossy().to_string());

            match record.file_type {
                FileType::Lockfile => {
                    dep.add_classification(
                        Classification::Should,
                        record.version.clone(),
                        record.source_file.clone(),
                    );
                }
                FileType::Manifest => {
                    dep.add_classification(
                        Classification::Can,
                        record.version.clone(),
                        record.source_file.clone(),
                    );
                }
            }

            results.push(dep);
        }

        results
    }
}

impl Default for Classifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{DependencyType, Ecosystem};
    use std::path::PathBuf;

    #[test]
    fn test_classify_has_only() {
        let classifier = Classifier::new();

        let installed = vec![InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        )];

        let classified = classifier.classify(vec![], installed);

        assert_eq!(classified.len(), 1);
        assert_eq!(classified[0].name, "react");
        assert!(classified[0].has_classification(Classification::Has));
        assert!(!classified[0].has_classification(Classification::Should));
        assert!(!classified[0].has_classification(Classification::Can));
        assert_eq!(
            classified[0].get_version(Classification::Has),
            Some("18.2.0")
        );
    }

    #[test]
    fn test_classify_should_only() {
        let classifier = Classifier::new();

        let records = vec![DependencyRecord {
            name: "react".to_string(),
            version: "18.2.0".to_string(),
            source_file: PathBuf::from("/app/package-lock.json"),
            dep_type: DependencyType::Runtime,
            ecosystem: Ecosystem::Node,
            file_type: FileType::Lockfile,
        }];

        let classified = classifier.classify(records, vec![]);

        assert_eq!(classified.len(), 1);
        assert!(!classified[0].has_classification(Classification::Has));
        assert!(classified[0].has_classification(Classification::Should));
        assert!(!classified[0].has_classification(Classification::Can));
        assert_eq!(
            classified[0].get_version(Classification::Should),
            Some("18.2.0")
        );
    }

    #[test]
    fn test_classify_can_only() {
        let classifier = Classifier::new();

        let records = vec![DependencyRecord {
            name: "react".to_string(),
            version: "^18.0.0".to_string(),
            source_file: PathBuf::from("/app/package.json"),
            dep_type: DependencyType::Runtime,
            ecosystem: Ecosystem::Node,
            file_type: FileType::Manifest,
        }];

        let classified = classifier.classify(records, vec![]);

        assert_eq!(classified.len(), 1);
        assert!(!classified[0].has_classification(Classification::Has));
        assert!(!classified[0].has_classification(Classification::Should));
        assert!(classified[0].has_classification(Classification::Can));
        assert_eq!(
            classified[0].get_version(Classification::Can),
            Some("^18.0.0")
        );
    }

    #[test]
    fn test_classify_all_three() {
        let classifier = Classifier::new();

        let installed = vec![InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        )];

        let records = vec![
            DependencyRecord {
                name: "react".to_string(),
                version: "18.2.0".to_string(),
                source_file: PathBuf::from("/app/package-lock.json"),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Lockfile,
            },
            DependencyRecord {
                name: "react".to_string(),
                version: "^18.0.0".to_string(),
                source_file: PathBuf::from("/app/package.json"),
                dep_type: DependencyType::Runtime,
                ecosystem: Ecosystem::Node,
                file_type: FileType::Manifest,
            },
        ];

        let classified = classifier.classify(records, installed);

        // Now each finding is separate - no deduplication
        assert_eq!(classified.len(), 3);

        // Find each classification
        let has_dep = classified
            .iter()
            .find(|d| d.has_classification(Classification::Has))
            .unwrap();
        let should_dep = classified
            .iter()
            .find(|d| d.has_classification(Classification::Should))
            .unwrap();
        let can_dep = classified
            .iter()
            .find(|d| d.has_classification(Classification::Can))
            .unwrap();

        assert_eq!(has_dep.get_version(Classification::Has), Some("18.2.0"));
        assert_eq!(
            should_dep.get_version(Classification::Should),
            Some("18.2.0")
        );
        assert_eq!(can_dep.get_version(Classification::Can), Some("^18.0.0"));
    }

    #[test]
    fn test_classify_multiple_packages() {
        let classifier = Classifier::new();

        let installed = vec![
            InstalledPackage::new(
                "react".to_string(),
                "18.2.0".to_string(),
                PathBuf::from("/app/node_modules/react"),
                Ecosystem::Node,
            ),
            InstalledPackage::new(
                "lodash".to_string(),
                "4.17.21".to_string(),
                PathBuf::from("/app/node_modules/lodash"),
                Ecosystem::Node,
            ),
        ];

        let classified = classifier.classify(vec![], installed);

        assert_eq!(classified.len(), 2);
        assert!(classified.iter().any(|d| d.name == "react"));
        assert!(classified.iter().any(|d| d.name == "lodash"));
    }

    #[test]
    fn test_classify_different_ecosystems() {
        let classifier = Classifier::new();

        let installed = vec![
            InstalledPackage::new(
                "react".to_string(),
                "18.2.0".to_string(),
                PathBuf::from("/app/node_modules/react"),
                Ecosystem::Node,
            ),
            InstalledPackage::new(
                "requests".to_string(),
                "2.31.0".to_string(),
                PathBuf::from("/app/site-packages/requests"),
                Ecosystem::Python,
            ),
        ];

        let classified = classifier.classify(vec![], installed);

        assert_eq!(classified.len(), 2);
        let react = classified.iter().find(|d| d.name == "react").unwrap();
        assert_eq!(react.ecosystem, Ecosystem::Node);
        let requests = classified.iter().find(|d| d.name == "requests").unwrap();
        assert_eq!(requests.ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_classify_stores_dependencies() {
        let classifier = Classifier::new();

        let mut pkg = InstalledPackage::new(
            "react".to_string(),
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
            Ecosystem::Node,
        );
        pkg.add_dependency("loose-envify".to_string(), "^1.1.0".to_string());
        pkg.add_dependency("scheduler".to_string(), "^0.23.0".to_string());

        let classified = classifier.classify(vec![], vec![pkg]);

        assert_eq!(classified.len(), 1);
        assert_eq!(classified[0].dependencies.len(), 2);
        assert!(classified[0]
            .dependencies
            .contains(&"loose-envify".to_string()));
        assert!(classified[0]
            .dependencies
            .contains(&"scheduler".to_string()));
    }

    #[test]
    fn test_classify_no_deduplication() {
        let classifier = Classifier::new();

        // Same package installed in two different locations
        let installed = vec![
            InstalledPackage::new(
                "react".to_string(),
                "18.2.0".to_string(),
                PathBuf::from("/app1/node_modules/react"),
                Ecosystem::Node,
            ),
            InstalledPackage::new(
                "react".to_string(),
                "18.2.0".to_string(),
                PathBuf::from("/app2/node_modules/react"),
                Ecosystem::Node,
            ),
        ];

        let classified = classifier.classify(vec![], installed);

        // Should have 2 separate entries, not deduplicated
        assert_eq!(classified.len(), 2);
        assert_eq!(classified[0].name, "react");
        assert_eq!(classified[1].name, "react");

        // Each should have different paths
        let paths: Vec<_> = classified
            .iter()
            .filter_map(|d| d.installed_path.as_ref())
            .collect();
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&&PathBuf::from("/app1/node_modules/react")));
        assert!(paths.contains(&&PathBuf::from("/app2/node_modules/react")));
    }
}
