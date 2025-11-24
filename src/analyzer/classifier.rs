//! Classifier component for assigning HAS/SHOULD/CAN classifications
//!
//! This module groups dependency records by package name and ecosystem,
//! then assigns appropriate classifications based on the source:
//!
//! - **HAS**: From installed package parsers (node_modules, site-packages)
//! - **SHOULD**: From lockfile parsers (package-lock.json, poetry.lock, etc.)
//! - **CAN**: From manifest parsers (package.json, pyproject.toml, etc.)

use crate::models::{
    Classification, ClassifiedDependency, DependencyRecord, Ecosystem, FileType, InstalledPackage,
};
use std::collections::HashMap;

/// Classifier for assigning HAS/SHOULD/CAN classifications
pub struct Classifier;

impl Classifier {
    /// Create a new Classifier
    pub fn new() -> Self {
        Self
    }

    /// Classify dependency records and installed packages
    ///
    /// Groups all dependencies by (name, ecosystem) and assigns appropriate
    /// classifications based on their source.
    pub fn classify(
        &self,
        records: Vec<DependencyRecord>,
        installed: Vec<InstalledPackage>,
    ) -> Vec<ClassifiedDependency> {
        // Group by (name, ecosystem)
        let mut grouped: HashMap<(String, Ecosystem), ClassifiedDependency> = HashMap::new();

        // Process installed packages (HAS classification)
        for pkg in installed {
            let key = (pkg.name.clone(), pkg.ecosystem);
            let dep = grouped
                .entry(key.clone())
                .or_insert_with(|| ClassifiedDependency::new(pkg.name.clone(), pkg.ecosystem));

            dep.add_classification(Classification::Has, pkg.version.clone(), pkg.path.clone());
            dep.installed_path = Some(pkg.path.clone());

            // Store dependencies for tree building
            for dep_spec in &pkg.dependencies {
                dep.dependencies.push(dep_spec.name.clone());
            }
        }

        // Process dependency records (SHOULD and CAN classifications)
        for record in records {
            let key = (record.name.clone(), record.ecosystem);
            let dep = grouped.entry(key.clone()).or_insert_with(|| {
                ClassifiedDependency::new(record.name.clone(), record.ecosystem)
            });

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
        }

        // Convert to vector
        grouped.into_values().collect()
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
    use crate::models::DependencyType;
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

        assert_eq!(classified.len(), 1);
        assert!(classified[0].has_classification(Classification::Has));
        assert!(classified[0].has_classification(Classification::Should));
        assert!(classified[0].has_classification(Classification::Can));
        assert_eq!(
            classified[0].get_version(Classification::Has),
            Some("18.2.0")
        );
        assert_eq!(
            classified[0].get_version(Classification::Should),
            Some("18.2.0")
        );
        assert_eq!(
            classified[0].get_version(Classification::Can),
            Some("^18.0.0")
        );
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
}
