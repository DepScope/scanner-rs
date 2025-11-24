//! Filesystem indexing and file discovery
//!
//! This module handles recursive directory traversal to identify package management files.

use std::path::Path;
use std::sync::Mutex;
use rayon::prelude::*;
use walkdir::WalkDir;

pub mod file_types;

pub use file_types::{classify_file, DiscoveredFile};

/// Find all package management files in a directory tree
pub fn find_files(root: &Path, exclude_dirs: &[&str]) -> Vec<DiscoveredFile> {
    // Collect all entries first (walkdir doesn't support parallel iteration directly)
    let entries: Vec<_> = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| !should_exclude(e.path(), exclude_dirs))
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
