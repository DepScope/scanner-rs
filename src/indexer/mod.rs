//! Filesystem indexing and file discovery
//!
//! This module handles recursive directory traversal to identify package management files.

use rayon::prelude::*;
use std::path::Path;
use std::sync::Mutex;
use walkdir::WalkDir;

pub mod file_types;
pub mod install_dirs;

pub use file_types::{classify_file, DiscoveredFile};
pub use install_dirs::{
    find_all_install_dirs, find_node_modules, find_site_packages, find_virtual_envs, InstallDir,
    InstallDirType,
};

/// Scan mode for directory traversal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScanMode {
    /// Scan all files (manifests, lockfiles, and installed packages)
    Full,
    /// Only scan installed packages (node_modules, site-packages)
    InstalledOnly,
    /// Only scan declared dependencies (manifests and lockfiles)
    DeclaredOnly,
}

/// Find all package management files in a directory tree
pub fn find_files(root: &Path, exclude_dirs: &[&str]) -> Vec<DiscoveredFile> {
    find_files_with_mode(root, exclude_dirs, ScanMode::Full, false)
}

/// Find all package management files with specified scan mode
pub fn find_files_with_mode(
    root: &Path,
    exclude_dirs: &[&str],
    scan_mode: ScanMode,
    include_install_dirs: bool,
) -> Vec<DiscoveredFile> {
    match scan_mode {
        ScanMode::Full => {
            // Scan both declared and installed
            find_declared_files(root, exclude_dirs, include_install_dirs)
        }
        ScanMode::InstalledOnly => {
            // Only scan installation directories - no manifest/lockfile parsing
            Vec::new() // Installed packages are handled separately via find_all_install_dirs
        }
        ScanMode::DeclaredOnly => {
            // Only scan manifests and lockfiles
            find_declared_files(root, exclude_dirs, include_install_dirs)
        }
    }
}

/// Find declared dependency files (manifests and lockfiles)
fn find_declared_files(
    root: &Path,
    exclude_dirs: &[&str],
    include_install_dirs: bool,
) -> Vec<DiscoveredFile> {
    // Build exclusion list
    let mut exclusions = exclude_dirs.to_vec();

    // Add installation directories to exclusions unless explicitly included
    if !include_install_dirs {
        exclusions.extend_from_slice(&[
            "node_modules",
            "site-packages",
            "dist-packages",
            ".venv",
            "venv",
            "env",
        ]);
    }
    // Collect all entries first (walkdir doesn't support parallel iteration directly)
    let entries: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), &exclusions))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();

    // Process entries in parallel
    let discovered = Mutex::new(Vec::new());

    entries.par_iter().for_each(|entry| {
        let file_name = entry.file_name().to_string_lossy();

        if let Some((ecosystem, file_type)) = classify_file(&file_name) {
            if let Some(parent) = entry.path().parent() {
                let file = DiscoveredFile {
                    path: entry.path().to_path_buf(),
                    filename: file_name.to_string(),
                    directory: parent.to_path_buf(),
                    ecosystem,
                    file_type,
                };
                discovered.lock().unwrap().push(file);
            }
        }
    });

    discovered.into_inner().unwrap()
}

/// Check if a path should be excluded from traversal
fn should_exclude(path: &Path, exclude_dirs: &[&str]) -> bool {
    path.components().any(|component| {
        if let Some(name) = component.as_os_str().to_str() {
            exclude_dirs.contains(&name)
        } else {
            false
        }
    })
}
