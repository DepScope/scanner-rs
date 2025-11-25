//! Infected package filter for matching dependencies against infected package lists
//!
//! This module filters classified dependencies to identify matches with
//! known infected packages (ransomware/worm) and sorts them by priority (HAS > SHOULD > CAN).

use crate::models::{Classification, ClassifiedDependency, ScanError};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// An infected package specification with multiple versions
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InfectedPackage {
    /// Package name
    pub name: String,
    /// Infected versions (empty set means all versions are infected)
    pub versions: HashSet<String>,
}

impl InfectedPackage {
    /// Create a new infected package with versions
    pub fn new(name: String, versions: HashSet<String>) -> Self {
        Self { name, versions }
    }

    /// Check if this infected package matches a dependency
    pub fn matches(&self, dep: &ClassifiedDependency) -> bool {
        if dep.name != self.name {
            return false;
        }

        // If no versions specified, match any version
        if self.versions.is_empty() {
            return true;
        }

        // Use primary version (Has > Should > Can) for matching
        if let Some(dep_version) = dep.get_primary_version() {
            if self.versions.contains(dep_version) {
                return true;
            }
        }

        false
    }

    /// Get the matched version from a dependency
    pub fn get_matched_version(&self, dep: &ClassifiedDependency) -> Option<String> {
        // Use primary version (Has > Should > Can) for matching
        if let Some(dep_version) = dep.get_primary_version() {
            if self.versions.is_empty() || self.versions.contains(dep_version) {
                return Some(dep_version.to_string());
            }
        }
        None
    }
}

/// Infected package filter for matching and sorting dependencies
pub struct InfectedPackageFilter {
    infected_packages: HashMap<String, InfectedPackage>,
}

impl InfectedPackageFilter {
    /// Create a new InfectedPackageFilter
    pub fn new() -> Self {
        Self {
            infected_packages: HashMap::new(),
        }
    }

    /// Load infected packages from a CSV file
    ///
    /// CSV format: package,version1 | version2 | version3
    /// Example:
    /// webpack-loader-httpfile,0.2.1
    /// zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1
    pub fn load_from_csv(&mut self, path: &Path) -> Result<(), ScanError> {
        let content = fs::read_to_string(path).map_err(ScanError::Io)?;

        for (line_num, line) in content.lines().enumerate() {
            let line = line.trim();
            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse CSV line: package,version1 | version2 | version3
            let parts: Vec<&str> = line.splitn(2, ',').collect();
            if parts.len() != 2 {
                return Err(ScanError::Parse {
                    file: path.to_path_buf(),
                    message: format!(
                        "Invalid CSV format at line {}: expected 'package,versions'",
                        line_num + 1
                    ),
                });
            }

            let package_name = parts[0].trim().to_string();
            let versions_str = parts[1].trim();

            // Parse versions separated by |
            let versions: HashSet<String> = versions_str
                .split('|')
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .collect();

            let infected = InfectedPackage::new(package_name.clone(), versions);
            self.infected_packages.insert(package_name, infected);
        }

        Ok(())
    }

    /// Add an infected package manually
    pub fn add_infected_package(&mut self, infected: InfectedPackage) {
        self.infected_packages
            .insert(infected.name.clone(), infected);
    }

    /// Filter dependencies to only include infected ones
    pub fn filter(&self, dependencies: Vec<ClassifiedDependency>) -> Vec<ClassifiedDependency> {
        dependencies
            .into_iter()
            .filter(|dep| self.is_infected(dep))
            .collect()
    }

    /// Check if a dependency is infected (exact match in HAS or SHOULD)
    pub fn is_infected(&self, dep: &ClassifiedDependency) -> bool {
        matches!(self.get_security_status(dep), SecurityStatus::Infected)
    }

    /// Get the security status for a dependency
    pub fn get_security_status(&self, dep: &ClassifiedDependency) -> SecurityStatus {
        if let Some(infected) = self.infected_packages.get(&dep.name) {
            // Check HAS (installed) - exact match = INFECTED
            if let Some(has_version) = dep.get_version(Classification::Has) {
                if infected.versions.is_empty() || infected.versions.contains(has_version) {
                    return SecurityStatus::Infected;
                }
            }

            // Check SHOULD (lockfile) - exact match = INFECTED
            if let Some(should_version) = dep.get_version(Classification::Should) {
                if infected.versions.is_empty() || infected.versions.contains(should_version) {
                    return SecurityStatus::Infected;
                }
            }

            // Check CAN (manifest/semver range) - could match = MATCH_VERSION
            if let Some(can_version) = dep.get_version(Classification::Can) {
                // Check if any infected version could satisfy the semver range
                if self.semver_could_match(can_version, &infected.versions, dep.ecosystem) {
                    return SecurityStatus::MatchVersion;
                }
            }

            // Package name matches but no version match
            SecurityStatus::MatchPackage
        } else {
            SecurityStatus::None
        }
    }

    /// Check if a semver range could match any of the infected versions
    fn semver_could_match(
        &self,
        range: &str,
        infected_versions: &HashSet<String>,
        ecosystem: crate::models::Ecosystem,
    ) -> bool {
        use crate::analyzer::VersionMatcher;

        // If no specific versions listed, any range could match
        if infected_versions.is_empty() {
            return true;
        }

        let matcher = VersionMatcher::new();

        // Check if any infected version satisfies the range
        for infected_version in infected_versions {
            match matcher.satisfies_range(infected_version, range, ecosystem) {
                Ok(true) => return true,
                _ => continue,
            }
        }

        false
    }

    /// Filter and sort by priority (HAS > SHOULD > CAN)
    pub fn filter_and_sort(
        &self,
        mut dependencies: Vec<ClassifiedDependency>,
    ) -> Vec<ClassifiedDependency> {
        // Filter to infected only
        dependencies.retain(|dep| self.is_infected(dep));

        // Sort by priority
        dependencies.sort_by(|a, b| {
            let a_priority = self.get_priority(a);
            let b_priority = self.get_priority(b);

            // Lower number = higher priority
            a_priority
                .cmp(&b_priority)
                .then_with(|| a.name.cmp(&b.name))
        });

        dependencies
    }

    /// Get priority for sorting (lower = higher priority)
    fn get_priority(&self, dep: &ClassifiedDependency) -> u8 {
        if dep.has_classification(Classification::Has) {
            0 // Highest priority
        } else if dep.has_classification(Classification::Should) {
            1
        } else if dep.has_classification(Classification::Can) {
            2
        } else {
            3 // Lowest priority
        }
    }

    /// Get the number of loaded infected packages
    pub fn count(&self) -> usize {
        self.infected_packages.len()
    }
}

impl Default for InfectedPackageFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Security status for a dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityStatus {
    /// No security issues - package not on infected list
    None,
    /// Package name matches infected list but version doesn't match
    MatchPackage,
    /// Semver range (CAN) could include an infected version
    MatchVersion,
    /// Exact version match in HAS or SHOULD (installed or locked)
    Infected,
}

impl SecurityStatus {
    /// Get priority for sorting (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            SecurityStatus::Infected => 0,
            SecurityStatus::MatchVersion => 1,
            SecurityStatus::MatchPackage => 2,
            SecurityStatus::None => 3,
        }
    }
}

impl std::fmt::Display for SecurityStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityStatus::None => write!(f, "NONE"),
            SecurityStatus::MatchPackage => write!(f, "MATCH_PACKAGE"),
            SecurityStatus::MatchVersion => write!(f, "MATCH_VERSION"),
            SecurityStatus::Infected => write!(f, "INFECTED"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Ecosystem;
    use std::path::PathBuf;
    use tempfile::NamedTempFile;

    #[test]
    fn test_infected_package_single_version() {
        let mut versions = HashSet::new();
        versions.insert("0.2.1".to_string());
        let infected = InfectedPackage::new("webpack-loader-httpfile".to_string(), versions);

        let mut dep =
            ClassifiedDependency::new("webpack-loader-httpfile".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "0.2.1".to_string(),
            PathBuf::from("/app/node_modules/webpack-loader-httpfile"),
        );

        assert!(infected.matches(&dep));
    }

    #[test]
    fn test_infected_package_multiple_versions() {
        let mut versions = HashSet::new();
        versions.insert("1.0.3".to_string());
        versions.insert("1.0.2".to_string());
        versions.insert("1.0.1".to_string());
        let infected = InfectedPackage::new("zapier-async-storage".to_string(), versions);

        let mut dep =
            ClassifiedDependency::new("zapier-async-storage".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "1.0.2".to_string(),
            PathBuf::from("/app/node_modules/zapier-async-storage"),
        );

        assert!(infected.matches(&dep));
    }

    #[test]
    fn test_no_match_different_version() {
        let mut versions = HashSet::new();
        versions.insert("1.0.3".to_string());
        let infected = InfectedPackage::new("zapier-async-storage".to_string(), versions);

        let mut dep =
            ClassifiedDependency::new("zapier-async-storage".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "1.0.4".to_string(),
            PathBuf::from("/app/node_modules/zapier-async-storage"),
        );

        assert!(!infected.matches(&dep));
    }

    #[test]
    fn test_load_from_csv() {
        let mut temp_file = NamedTempFile::new().unwrap();
        use std::io::Write;
        writeln!(temp_file, "webpack-loader-httpfile,0.2.1").unwrap();
        writeln!(temp_file, "wellness-expert-ng-gallery,5.1.1").unwrap();
        writeln!(temp_file, "zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1").unwrap();
        writeln!(temp_file, "# comment").unwrap();
        writeln!(temp_file).unwrap();
        writeln!(temp_file, "zapier-platform-cli,18.0.4 | 18.0.3 | 18.0.2").unwrap();
        temp_file.flush().unwrap();

        let mut filter = InfectedPackageFilter::new();
        filter.load_from_csv(temp_file.path()).unwrap();

        assert_eq!(filter.count(), 4);
    }

    #[test]
    fn test_filter() {
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("0.2.1".to_string());
        filter.add_infected_package(InfectedPackage::new(
            "webpack-loader-httpfile".to_string(),
            versions,
        ));

        let mut dep1 =
            ClassifiedDependency::new("webpack-loader-httpfile".to_string(), Ecosystem::Node);
        dep1.add_classification(
            Classification::Has,
            "0.2.1".to_string(),
            PathBuf::from("/app/node_modules/webpack-loader-httpfile"),
        );

        let mut dep2 = ClassifiedDependency::new("lodash".to_string(), Ecosystem::Node);
        dep2.add_classification(
            Classification::Has,
            "4.17.21".to_string(),
            PathBuf::from("/app/node_modules/lodash"),
        );

        let filtered = filter.filter(vec![dep1, dep2]);

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "webpack-loader-httpfile");
    }

    #[test]
    fn test_security_status_none() {
        let filter = InfectedPackageFilter::new();

        let mut dep = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        assert_eq!(filter.get_security_status(&dep), SecurityStatus::None);
    }

    #[test]
    fn test_security_status_match_package() {
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("1.0.1".to_string());
        filter.add_infected_package(InfectedPackage::new(
            "zapier-async-storage".to_string(),
            versions,
        ));

        let mut dep =
            ClassifiedDependency::new("zapier-async-storage".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "1.0.4".to_string(),
            PathBuf::from("/app/node_modules/zapier-async-storage"),
        );

        assert_eq!(
            filter.get_security_status(&dep),
            SecurityStatus::MatchPackage
        );
    }

    #[test]
    fn test_security_status_match_version_semver() {
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("1.0.1".to_string());
        filter.add_infected_package(InfectedPackage::new(
            "zapier-async-storage".to_string(),
            versions,
        ));

        let mut dep =
            ClassifiedDependency::new("zapier-async-storage".to_string(), Ecosystem::Node);
        // Semver range that includes 1.0.1
        dep.add_classification(
            Classification::Can,
            "^1.0.0".to_string(),
            PathBuf::from("/app/package.json"),
        );

        assert_eq!(
            filter.get_security_status(&dep),
            SecurityStatus::MatchVersion
        );
    }

    #[test]
    fn test_security_status_infected() {
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("1.0.1".to_string());
        filter.add_infected_package(InfectedPackage::new(
            "zapier-async-storage".to_string(),
            versions,
        ));

        let mut dep =
            ClassifiedDependency::new("zapier-async-storage".to_string(), Ecosystem::Node);
        dep.add_classification(
            Classification::Has,
            "1.0.1".to_string(),
            PathBuf::from("/app/node_modules/zapier-async-storage"),
        );

        assert_eq!(filter.get_security_status(&dep), SecurityStatus::Infected);
    }

    #[test]
    fn test_filter_and_sort_by_priority() {
        let mut filter = InfectedPackageFilter::new();
        let mut versions = HashSet::new();
        versions.insert("18.2.0".to_string());
        filter.add_infected_package(InfectedPackage::new("react".to_string(), versions));

        // Create deps with different classifications
        let mut dep_has = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep_has.add_classification(
            Classification::Has,
            "18.2.0".to_string(),
            PathBuf::from("/app/node_modules/react"),
        );

        let mut dep_should = ClassifiedDependency::new("react".to_string(), Ecosystem::Node);
        dep_should.add_classification(
            Classification::Should,
            "18.2.0".to_string(),
            PathBuf::from("/app/package-lock.json"),
        );

        // Add in reverse priority order
        let sorted = filter.filter_and_sort(vec![dep_should, dep_has]);

        // Only HAS and SHOULD are INFECTED (exact match), CAN would be MATCH_VERSION
        assert_eq!(sorted.len(), 2);
        // HAS should be first
        assert!(sorted[0].has_classification(Classification::Has));
        // SHOULD should be second
        assert!(sorted[1].has_classification(Classification::Should));
    }
}
