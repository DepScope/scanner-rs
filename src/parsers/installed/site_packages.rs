//! Parser for Python installed packages in site-packages directories

use super::metadata::{parse_metadata_file, parse_pkg_info_file};
use crate::models::error::ScanError;
use crate::models::{Ecosystem, InstalledPackage};
use std::fs;
use std::path::Path;

/// Parser for site-packages directories
pub struct SitePackagesParser;

impl SitePackagesParser {
    /// Parse all installed packages in a site-packages directory
    pub fn parse_installed(
        &self,
        site_packages_path: &Path,
    ) -> Result<Vec<InstalledPackage>, ScanError> {
        let mut packages = Vec::new();

        // Read all entries in site-packages
        let entries = fs::read_dir(site_packages_path).map_err(ScanError::Io)?;

        for entry in entries {
            let entry = entry.map_err(ScanError::Io)?;
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            // Check for .dist-info directories (modern format)
            if path.is_dir() && name_str.ends_with(".dist-info") {
                if let Ok(pkg) = self.parse_dist_info(&path) {
                    packages.push(pkg);
                }
            }
            // Check for .egg-info directories (legacy format)
            else if path.is_dir() && name_str.ends_with(".egg-info") {
                if let Ok(pkg) = self.parse_egg_info_dir(&path) {
                    packages.push(pkg);
                }
            }
            // Check for .egg-info files (even older format)
            else if path.is_file() && name_str.ends_with(".egg-info") {
                if let Ok(pkg) = self.parse_egg_info_file(&path) {
                    packages.push(pkg);
                }
            }
        }

        Ok(packages)
    }

    /// Parse a .dist-info directory
    fn parse_dist_info(&self, dist_info_path: &Path) -> Result<InstalledPackage, ScanError> {
        let metadata_path = dist_info_path.join("METADATA");

        if !metadata_path.exists() {
            return Err(ScanError::Parse {
                file: metadata_path.clone(),
                message: "METADATA file not found in .dist-info directory".to_string(),
            });
        }

        let metadata = parse_metadata_file(&metadata_path)?;

        // The package directory is typically the parent of .dist-info
        let package_path = dist_info_path
            .parent()
            .ok_or_else(|| ScanError::Parse {
                file: dist_info_path.to_path_buf(),
                message: "Could not determine package path".to_string(),
            })?
            .join(&metadata.name);

        let mut package = InstalledPackage::new(
            metadata.name,
            metadata.version,
            package_path,
            Ecosystem::Python,
        );

        // Add dependencies
        for (dep_name, dep_version) in metadata.dependencies {
            package.add_dependency(dep_name, dep_version);
        }

        Ok(package)
    }

    /// Parse a .egg-info directory
    fn parse_egg_info_dir(&self, egg_info_path: &Path) -> Result<InstalledPackage, ScanError> {
        let pkg_info_path = egg_info_path.join("PKG-INFO");

        if !pkg_info_path.exists() {
            return Err(ScanError::Parse {
                file: pkg_info_path.clone(),
                message: "PKG-INFO file not found in .egg-info directory".to_string(),
            });
        }

        let metadata = parse_pkg_info_file(&pkg_info_path)?;

        // The package directory is typically the parent of .egg-info
        let package_path = egg_info_path
            .parent()
            .ok_or_else(|| ScanError::Parse {
                file: egg_info_path.to_path_buf(),
                message: "Could not determine package path".to_string(),
            })?
            .join(&metadata.name);

        let mut package = InstalledPackage::new(
            metadata.name,
            metadata.version,
            package_path,
            Ecosystem::Python,
        );

        // Add dependencies
        for (dep_name, dep_version) in metadata.dependencies {
            package.add_dependency(dep_name, dep_version);
        }

        Ok(package)
    }

    /// Parse a .egg-info file (single file, not directory)
    fn parse_egg_info_file(&self, egg_info_path: &Path) -> Result<InstalledPackage, ScanError> {
        let metadata = parse_pkg_info_file(egg_info_path)?;

        // The package directory is typically the parent of .egg-info file
        let package_path = egg_info_path
            .parent()
            .ok_or_else(|| ScanError::Parse {
                file: egg_info_path.to_path_buf(),
                message: "Could not determine package path".to_string(),
            })?
            .join(&metadata.name);

        let mut package = InstalledPackage::new(
            metadata.name,
            metadata.version,
            package_path,
            Ecosystem::Python,
        );

        // Add dependencies
        for (dep_name, dep_version) in metadata.dependencies {
            package.add_dependency(dep_name, dep_version);
        }

        Ok(package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_parse_dist_info() {
        let temp_dir = TempDir::new().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create .dist-info directory
        let dist_info = site_packages.join("requests-2.31.0.dist-info");
        fs::create_dir_all(&dist_info).unwrap();

        let metadata = r#"Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Requires-Dist: charset-normalizer (<4,>=2)
Requires-Dist: urllib3 (<3,>=1.21.1)
"#;

        fs::write(dist_info.join("METADATA"), metadata).unwrap();

        let parser = SitePackagesParser;
        let packages = parser.parse_installed(&site_packages).unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "requests");
        assert_eq!(packages[0].version, "2.31.0");
        assert_eq!(packages[0].ecosystem, Ecosystem::Python);
        assert_eq!(packages[0].dependencies.len(), 2);
        assert_eq!(packages[0].dependencies[0].name, "charset-normalizer");
        assert_eq!(packages[0].dependencies[1].name, "urllib3");
    }

    #[test]
    fn test_parse_egg_info_dir() {
        let temp_dir = TempDir::new().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create .egg-info directory
        let egg_info = site_packages.join("simplejson-3.19.1.egg-info");
        fs::create_dir_all(&egg_info).unwrap();

        let pkg_info = r#"Metadata-Version: 1.1
Name: simplejson
Version: 3.19.1
"#;

        fs::write(egg_info.join("PKG-INFO"), pkg_info).unwrap();

        let parser = SitePackagesParser;
        let packages = parser.parse_installed(&site_packages).unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "simplejson");
        assert_eq!(packages[0].version, "3.19.1");
        assert_eq!(packages[0].ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_parse_egg_info_file() {
        let temp_dir = TempDir::new().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create .egg-info file (not directory)
        let pkg_info = r#"Metadata-Version: 1.0
Name: oldpackage
Version: 1.0.0
"#;

        fs::write(site_packages.join("oldpackage-1.0.0.egg-info"), pkg_info).unwrap();

        let parser = SitePackagesParser;
        let packages = parser.parse_installed(&site_packages).unwrap();

        assert_eq!(packages.len(), 1);
        assert_eq!(packages[0].name, "oldpackage");
        assert_eq!(packages[0].version, "1.0.0");
    }

    #[test]
    fn test_parse_multiple_packages() {
        let temp_dir = TempDir::new().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create multiple .dist-info directories
        let requests_dist_info = site_packages.join("requests-2.31.0.dist-info");
        fs::create_dir_all(&requests_dist_info).unwrap();
        fs::write(
            requests_dist_info.join("METADATA"),
            "Metadata-Version: 2.1\nName: requests\nVersion: 2.31.0\n",
        )
        .unwrap();

        let urllib3_dist_info = site_packages.join("urllib3-2.0.7.dist-info");
        fs::create_dir_all(&urllib3_dist_info).unwrap();
        fs::write(
            urllib3_dist_info.join("METADATA"),
            "Metadata-Version: 2.1\nName: urllib3\nVersion: 2.0.7\n",
        )
        .unwrap();

        let parser = SitePackagesParser;
        let packages = parser.parse_installed(&site_packages).unwrap();

        assert_eq!(packages.len(), 2);
        assert!(packages.iter().any(|p| p.name == "requests"));
        assert!(packages.iter().any(|p| p.name == "urllib3"));
    }

    #[test]
    fn test_parse_mixed_formats() {
        let temp_dir = TempDir::new().unwrap();
        let site_packages = temp_dir.path().join("site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        // Create .dist-info
        let dist_info = site_packages.join("requests-2.31.0.dist-info");
        fs::create_dir_all(&dist_info).unwrap();
        fs::write(
            dist_info.join("METADATA"),
            "Metadata-Version: 2.1\nName: requests\nVersion: 2.31.0\n",
        )
        .unwrap();

        // Create .egg-info directory
        let egg_info_dir = site_packages.join("simplejson-3.19.1.egg-info");
        fs::create_dir_all(&egg_info_dir).unwrap();
        fs::write(
            egg_info_dir.join("PKG-INFO"),
            "Metadata-Version: 1.1\nName: simplejson\nVersion: 3.19.1\n",
        )
        .unwrap();

        // Create .egg-info file
        fs::write(
            site_packages.join("oldpackage-1.0.0.egg-info"),
            "Metadata-Version: 1.0\nName: oldpackage\nVersion: 1.0.0\n",
        )
        .unwrap();

        let parser = SitePackagesParser;
        let packages = parser.parse_installed(&site_packages).unwrap();

        assert_eq!(packages.len(), 3);
        assert!(packages.iter().any(|p| p.name == "requests"));
        assert!(packages.iter().any(|p| p.name == "simplejson"));
        assert!(packages.iter().any(|p| p.name == "oldpackage"));
    }
}
