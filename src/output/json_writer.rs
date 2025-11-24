//! JSON output writer for dependency trees

use crate::analyzer::InfectedPackageFilter;
use crate::models::{Application, DependencyTree};
use serde_json;
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Write applications with classified dependencies to a JSON file
pub fn write_applications_json(
    applications: &[Application],
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    write_applications_json_with_security(applications.to_vec(), None, output_path)
}

/// Write applications with classified dependencies and security status to a JSON file
pub fn write_applications_json_with_security(
    applications: Vec<Application>,
    security_filter: Option<&InfectedPackageFilter>,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut apps = applications;

    // Add security status to all dependencies if filter is provided
    if let Some(filter) = security_filter {
        for app in &mut apps {
            for dep in &mut app.dependencies {
                dep.security = Some(filter.get_security_status(dep).to_string());
            }
        }
    }

    let json = serde_json::to_string_pretty(&apps)?;
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

/// Write dependency trees to a JSON file
pub fn write_trees_json(
    trees: &[DependencyTree],
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    write_trees_json_with_security(trees.to_vec(), None, output_path)
}

/// Write dependency trees with security status to a JSON file
pub fn write_trees_json_with_security(
    trees: Vec<DependencyTree>,
    security_filter: Option<&InfectedPackageFilter>,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut tree_vec = trees;

    // Add security status to all dependencies if filter is provided
    if let Some(filter) = security_filter {
        for tree in &mut tree_vec {
            for dep in &mut tree.application.dependencies {
                dep.security = Some(filter.get_security_status(dep).to_string());
            }
        }
    }

    let json = serde_json::to_string_pretty(&tree_vec)?;
    let mut file = File::create(output_path)?;
    file.write_all(json.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::InfectedPackageFilter;
    use crate::models::{Classification, ClassifiedDependency, Ecosystem};
    use std::collections::HashSet;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_write_applications_json() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        app.add_dependency(dep);

        let temp_file = NamedTempFile::new().unwrap();
        write_applications_json(&[app], temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("myapp"));
        assert!(content.contains("react"));
        assert!(content.contains("18.2.0"));
    }

    #[test]
    fn test_write_applications_json_with_security() {
        let mut app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        app.add_dependency(dep);

        // Create infected filter
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("18.2.0".to_string());
        filter.add_infected_package(crate::analyzer::vuln_filter::InfectedPackage::new(
            "react".to_string(),
            versions,
        ));

        let temp_file = NamedTempFile::new().unwrap();
        write_applications_json_with_security(vec![app], Some(&filter), temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("myapp"));
        assert!(content.contains("react"));
        assert!(content.contains("18.2.0"));
        assert!(content.contains("INFECTED"));
    }

    #[test]
    fn test_write_trees_json() {
        let app = Application::new(
            "myapp".to_string(),
            PathBuf::from("/app"),
            PathBuf::from("/app/package.json"),
            Ecosystem::Node,
        );

        let tree = DependencyTree::new(app);

        let temp_file = NamedTempFile::new().unwrap();
        write_trees_json(&[tree], temp_file.path()).unwrap();

        let content = std::fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("myapp"));
        assert!(content.contains("application"));
    }
}
