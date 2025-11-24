//! Installation directory detection for node_modules, site-packages, and virtual environments
//!
//! This module provides functions to detect and classify package installation directories
//! across different ecosystems:
//!
//! - **Node.js**: node_modules directories
//! - **Python**: site-packages, dist-packages, and virtual environments
//!
//! # Virtual Environment Detection
//!
//! Python virtual environments are detected through multiple methods:
//! - Presence of `pyvenv.cfg` file (definitive marker)
//! - Common directory names (.venv, venv, env) with activation scripts
//! - Automatic linking of site-packages to their parent virtual environment
//!
//! # Example
//!
//! ```rust
//! use scanner::indexer::{find_node_modules, find_site_packages, find_virtual_envs};
//! use std::path::Path;
//!
//! let root = Path::new("/path/to/project");
//! let exclude_dirs = &["target", ".git"];
//!
//! // Find all node_modules directories
//! let node_modules = find_node_modules(root, exclude_dirs);
//!
//! // Find all Python site-packages
//! let site_packages = find_site_packages(root, exclude_dirs);
//!
//! // Find all virtual environments
//! let venvs = find_virtual_envs(root, exclude_dirs);
//! ```

use crate::models::Ecosystem;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Type of installation directory
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallDirType {
    /// Node.js node_modules directory
    NodeModules,
    /// Python site-packages directory
    SitePackages,
    /// Python dist-packages directory
    DistPackages,
    /// Python virtual environment
    VirtualEnv,
}

/// A discovered installation directory
#[derive(Debug, Clone)]
pub struct InstallDir {
    /// Path to the installation directory
    pub path: PathBuf,

    /// Type of installation directory
    pub dir_type: InstallDirType,

    /// Ecosystem
    pub ecosystem: Ecosystem,

    /// Virtual environment root (if applicable)
    pub venv_root: Option<PathBuf>,
}

impl InstallDir {
    /// Create a new InstallDir
    pub fn new(path: PathBuf, dir_type: InstallDirType, ecosystem: Ecosystem) -> Self {
        Self {
            path,
            dir_type,
            ecosystem,
            venv_root: None,
        }
    }

    /// Set the virtual environment root
    pub fn with_venv_root(mut self, venv_root: PathBuf) -> Self {
        self.venv_root = Some(venv_root);
        self
    }
}

/// Find all node_modules directories in a directory tree
pub fn find_node_modules(root: &Path, exclude_dirs: &[&str]) -> Vec<InstallDir> {
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !should_exclude_for_install_scan(e.path(), exclude_dirs))
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if name == "node_modules" {
                    results.push(InstallDir::new(
                        entry.path().to_path_buf(),
                        InstallDirType::NodeModules,
                        Ecosystem::Node,
                    ));
                }
            }
        }
    }

    results
}

/// Find all site-packages and dist-packages directories in a directory tree
pub fn find_site_packages(root: &Path, exclude_dirs: &[&str]) -> Vec<InstallDir> {
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !should_exclude_for_install_scan(e.path(), exclude_dirs))
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                let (dir_type, is_match) = match name {
                    "site-packages" => (InstallDirType::SitePackages, true),
                    "dist-packages" => (InstallDirType::DistPackages, true),
                    _ => (InstallDirType::SitePackages, false),
                };

                if is_match {
                    let mut install_dir =
                        InstallDir::new(entry.path().to_path_buf(), dir_type, Ecosystem::Python);

                    // Check if this is within a virtual environment
                    if let Some(venv_root) = find_venv_root(entry.path()) {
                        install_dir = install_dir.with_venv_root(venv_root);
                    }

                    results.push(install_dir);
                }
            }
        }
    }

    results
}

/// Find all Python virtual environments in a directory tree
pub fn find_virtual_envs(root: &Path, exclude_dirs: &[&str]) -> Vec<InstallDir> {
    let mut results = Vec::new();

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !should_exclude_for_install_scan(e.path(), exclude_dirs))
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_dir() {
            // Check for pyvenv.cfg file (definitive marker of venv)
            let pyvenv_cfg = entry.path().join("pyvenv.cfg");
            if pyvenv_cfg.exists() {
                results.push(InstallDir::new(
                    entry.path().to_path_buf(),
                    InstallDirType::VirtualEnv,
                    Ecosystem::Python,
                ));
                continue;
            }

            // Check for common venv directory names
            if let Some(name) = entry.file_name().to_str() {
                if matches!(name, ".venv" | "venv" | "env") {
                    // Verify it looks like a venv (has bin/activate or Scripts/activate.bat)
                    let has_activate = entry.path().join("bin/activate").exists()
                        || entry.path().join("Scripts/activate.bat").exists();

                    if has_activate {
                        results.push(InstallDir::new(
                            entry.path().to_path_buf(),
                            InstallDirType::VirtualEnv,
                            Ecosystem::Python,
                        ));
                    }
                }
            }
        }
    }

    results
}

/// Find the virtual environment root for a given path
fn find_venv_root(path: &Path) -> Option<PathBuf> {
    let mut current = path;

    while let Some(parent) = current.parent() {
        // Check for pyvenv.cfg
        if parent.join("pyvenv.cfg").exists() {
            return Some(parent.to_path_buf());
        }

        // Check for common venv structure
        if let Some(name) = parent.file_name().and_then(|n| n.to_str()) {
            if matches!(name, ".venv" | "venv" | "env") {
                let has_activate = parent.join("bin/activate").exists()
                    || parent.join("Scripts/activate.bat").exists();

                if has_activate {
                    return Some(parent.to_path_buf());
                }
            }
        }

        current = parent;
    }

    None
}

/// Check if a path should be excluded from installation directory scanning
fn should_exclude_for_install_scan(path: &Path, exclude_dirs: &[&str]) -> bool {
    // For installation scanning, we want to find node_modules and site-packages,
    // but we don't want to traverse INTO them (to avoid nested scans)
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        // Check custom exclusions
        if exclude_dirs.contains(&name) {
            return true;
        }

        // Special handling for installation directories:
        // We want to discover them but not traverse into them
        // Note: This function is used with filter_entry which is called BEFORE
        // yielding the entry, so we need to allow the directory itself through
        // but prevent descending into it. However, filter_entry doesn't distinguish
        // between "yield but don't descend" - it's all or nothing.
        // So we allow these through and rely on the fact that we only care about
        // the top-level directory, not its contents.
    }

    false
}

/// Find all installation directories (convenience function)
pub fn find_all_install_dirs(root: &Path, exclude_dirs: &[&str]) -> Vec<InstallDir> {
    let mut results = Vec::new();

    results.extend(find_node_modules(root, exclude_dirs));
    results.extend(find_site_packages(root, exclude_dirs));
    results.extend(find_virtual_envs(root, exclude_dirs));

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_install_dir_creation() {
        let dir = InstallDir::new(
            PathBuf::from("/app/node_modules"),
            InstallDirType::NodeModules,
            Ecosystem::Node,
        );

        assert_eq!(dir.path, PathBuf::from("/app/node_modules"));
        assert_eq!(dir.dir_type, InstallDirType::NodeModules);
        assert_eq!(dir.ecosystem, Ecosystem::Node);
        assert!(dir.venv_root.is_none());
    }

    #[test]
    fn test_install_dir_with_venv_root() {
        let dir = InstallDir::new(
            PathBuf::from("/app/.venv/lib/python3.11/site-packages"),
            InstallDirType::SitePackages,
            Ecosystem::Python,
        )
        .with_venv_root(PathBuf::from("/app/.venv"));

        assert_eq!(dir.venv_root, Some(PathBuf::from("/app/.venv")));
    }

    #[test]
    fn test_find_node_modules() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create node_modules directory
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::create_dir_all(root.join("src/nested/node_modules")).unwrap();

        let results = find_node_modules(root, &[]);

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .all(|d| d.dir_type == InstallDirType::NodeModules));
        assert!(results.iter().all(|d| d.ecosystem == Ecosystem::Node));
    }

    #[test]
    fn test_find_site_packages() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create site-packages directory
        fs::create_dir_all(root.join("lib/python3.11/site-packages")).unwrap();
        fs::create_dir_all(root.join("lib/python3.10/dist-packages")).unwrap();

        let results = find_site_packages(root, &[]);

        assert_eq!(results.len(), 2);
        assert!(results
            .iter()
            .any(|d| d.dir_type == InstallDirType::SitePackages));
        assert!(results
            .iter()
            .any(|d| d.dir_type == InstallDirType::DistPackages));
        assert!(results.iter().all(|d| d.ecosystem == Ecosystem::Python));
    }

    #[test]
    fn test_find_virtual_envs_with_pyvenv_cfg() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create venv with pyvenv.cfg
        let venv_path = root.join(".venv");
        fs::create_dir_all(&venv_path).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "home = /usr/bin\n").unwrap();

        let results = find_virtual_envs(root, &[]);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dir_type, InstallDirType::VirtualEnv);
        assert_eq!(results[0].ecosystem, Ecosystem::Python);
    }

    #[test]
    fn test_find_virtual_envs_with_activate() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create venv with bin/activate
        let venv_path = root.join("venv");
        fs::create_dir_all(venv_path.join("bin")).unwrap();
        fs::write(venv_path.join("bin/activate"), "#!/bin/bash\n").unwrap();

        let results = find_virtual_envs(root, &[]);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].dir_type, InstallDirType::VirtualEnv);
    }

    #[test]
    fn test_find_venv_root() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create venv structure
        let venv_path = root.join(".venv");
        fs::create_dir_all(&venv_path).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "home = /usr/bin\n").unwrap();

        let site_packages = venv_path.join("lib/python3.11/site-packages");
        fs::create_dir_all(&site_packages).unwrap();

        let venv_root = find_venv_root(&site_packages);
        assert_eq!(venv_root, Some(venv_path));
    }

    #[test]
    fn test_find_all_install_dirs() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create various installation directories
        fs::create_dir_all(root.join("node_modules")).unwrap();
        fs::create_dir_all(root.join("lib/python3.11/site-packages")).unwrap();

        let venv_path = root.join(".venv");
        fs::create_dir_all(&venv_path).unwrap();
        fs::write(venv_path.join("pyvenv.cfg"), "home = /usr/bin\n").unwrap();

        let results = find_all_install_dirs(root, &[]);

        // Should find node_modules, site-packages, and venv
        assert!(results.len() >= 3);
        assert!(results
            .iter()
            .any(|d| d.dir_type == InstallDirType::NodeModules));
        assert!(results
            .iter()
            .any(|d| d.dir_type == InstallDirType::SitePackages));
        assert!(results
            .iter()
            .any(|d| d.dir_type == InstallDirType::VirtualEnv));
    }
}
