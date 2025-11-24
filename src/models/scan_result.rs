//! Scan result aggregation

use std::collections::HashMap;
use crate::models::{DependencyRecord, Ecosystem};

/// Aggregated scan results
#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    /// All discovered dependencies
    pub dependencies: Vec<DependencyRecord>,
}

impl ScanResult {
    /// Create a new empty scan result
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }
    
    /// Add a dependency record
    pub fn add(&mut self, record: DependencyRecord) {
        self.dependencies.push(record);
    }
    
    /// Add multiple dependency records
    pub fn add_all(&mut self, records: Vec<DependencyRecord>) {
        self.dependencies.extend(records);
    }
    
    /// Get total number of dependencies
    pub fn total_count(&self) -> usize {
        self.dependencies.len()
    }
    
    /// Get dependencies by ecosystem
    pub fn by_ecosystem(&self, ecosystem: Ecosystem) -> Vec<&DependencyRecord> {
        self.dependencies
            .iter()
            .filter(|d| d.ecosystem == ecosystem)
            .collect()
    }
    
    /// Get dependencies by package name
    pub fn by_package(&self, name: &str) -> Vec<&DependencyRecord> {
        self.dependencies
            .iter()
            .filter(|d| d.name == name)
            .collect()
    }
    
    /// Get unique package names
    pub fn unique_packages(&self) -> Vec<String> {
        let mut packages: Vec<String> = self.dependencies
            .iter()
            .map(|d| d.name.clone())
            .collect();
        packages.sort();
        packages.dedup();
        packages
    }
    
    /// Get statistics by ecosystem
    pub fn ecosystem_stats(&self) -> HashMap<Ecosystem, usize> {
        let mut stats = HashMap::new();
        for dep in &self.dependencies {
            *stats.entry(dep.ecosystem).or_insert(0) += 1;
        }
        stats
    }
    
    /// Sort dependencies by ecosystem, package name, and source file
    pub fn sort(&mut self) {
        self.dependencies.sort_by(|a, b| {
            a.ecosystem.to_string()
                .cmp(&b.ecosystem.to_string())
                .then_with(|| a.name.cmp(&b.name))
                .then_with(|| a.source_file.cmp(&b.source_file))
        });
    }
}
