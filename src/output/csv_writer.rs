//! CSV output writer

use csv::Writer;
use std::path::Path;

use crate::models::{Classification, ClassifiedDependency, DependencyRecord};

/// Write dependency records to a CSV file (legacy format)
pub fn write_csv(
    records: &[DependencyRecord],
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut writer = Writer::from_path(output_path)?;

    // Write header
    writer.write_record([
        "package",
        "version",
        "source_file",
        "dep_type",
        "ecosystem",
        "file_type",
    ])?;

    // Write records
    for record in records {
        writer.write_record([
            &record.name,
            &record.version,
            record.source_file.to_string_lossy().as_ref(),
            &record.dep_type.to_string(),
            &record.ecosystem.to_string(),
            &record.file_type.to_string(),
        ])?;
    }

    writer.flush()?;
    Ok(())
}

/// Write classified dependencies to a CSV file (enhanced format)
pub fn write_classified_csv(
    dependencies: &[ClassifiedDependency],
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    write_classified_csv_with_security(dependencies, None, output_path)
}

/// Write classified dependencies to a CSV file with security status
pub fn write_classified_csv_with_security(
    dependencies: &[ClassifiedDependency],
    security_filter: Option<&crate::analyzer::InfectedPackageFilter>,
    output_path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let mut writer = Writer::from_path(output_path)?;

    // Write header
    writer.write_record([
        "package_name",
        "package_name_path",
        "version",
        "ecosystem",
        "application_name",
        "application_root",
        "has_version",
        "has_path",
        "should_version",
        "should_path",
        "can_version",
        "can_path",
        "version_mismatch",
        "constraint_violation",
        "parent_package",
        "is_direct",
        "dependency_count",
        "security",
    ])?;

    // Write records
    for dep in dependencies {
        let has_version = dep
            .get_version(Classification::Has)
            .unwrap_or("")
            .to_string();
        let has_path = dep
            .get_source_file(Classification::Has)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let should_version = dep
            .get_version(Classification::Should)
            .unwrap_or("")
            .to_string();
        let should_path = dep
            .get_source_file(Classification::Should)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let can_version = dep
            .get_version(Classification::Can)
            .unwrap_or("")
            .to_string();
        let can_path = dep
            .get_source_file(Classification::Can)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let application_name = dep.application_name.as_deref().unwrap_or("");
        let application_root = dep
            .application_root
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();

        let parent_package = dep.parent_package.as_deref().unwrap_or("");
        let is_direct = if dep.parent_package.is_none() {
            "true"
        } else {
            "false"
        };

        let security = if let Some(filter) = security_filter {
            filter.get_security_status(dep).to_string()
        } else {
            "NONE".to_string()
        };

        let package_name_path = dep.package_name_path.as_deref().unwrap_or("");
        let version = dep.get_primary_version().unwrap_or("");

        writer.write_record([
            &dep.name,
            package_name_path,
            version,
            &dep.ecosystem.to_string(),
            application_name,
            &application_root,
            &has_version,
            &has_path,
            &should_version,
            &should_path,
            &can_version,
            &can_path,
            &dep.has_version_mismatch.to_string(),
            &dep.has_constraint_violation.to_string(),
            parent_package,
            is_direct,
            &dep.dependencies.len().to_string(),
            &security,
        ])?;
    }

    writer.flush()?;
    Ok(())
}
