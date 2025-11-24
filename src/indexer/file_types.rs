//! File type identification and classification

use std::path::PathBuf;

use crate::models::{Ecosystem, FileType};

/// A discovered package management file
#[derive(Debug, Clone)]
pub struct DiscoveredFile {
    /// Absolute path to the file
    pub path: PathBuf,
    /// File name (e.g., "package.json")
    pub filename: String,
    /// Parent directory path
    pub directory: PathBuf,
    /// Detected ecosystem
    pub ecosystem: Ecosystem,
    /// File type classification
    pub file_type: FileType,
}

/// Classify a file by its name and return its ecosystem and type
pub fn classify_file(filename: &str) -> Option<(Ecosystem, FileType)> {
    match filename {
        // Node.js manifest files
        "package.json" => Some((Ecosystem::Node, FileType::Manifest)),

        // Node.js lockfiles
        "yarn.lock" => Some((Ecosystem::Node, FileType::Lockfile)),
        "package-lock.json" => Some((Ecosystem::Node, FileType::Lockfile)),
        "pnpm-lock.yaml" => Some((Ecosystem::Node, FileType::Lockfile)),
        "bun.lock" => Some((Ecosystem::Node, FileType::Lockfile)),
        "npm-shrinkwrap.json" => Some((Ecosystem::Node, FileType::Lockfile)),

        // Python manifest files
        "pyproject.toml" => Some((Ecosystem::Python, FileType::Manifest)),
        "requirements.txt" => Some((Ecosystem::Python, FileType::Manifest)),
        "Pipfile" => Some((Ecosystem::Python, FileType::Manifest)),
        "environment.yml" => Some((Ecosystem::Python, FileType::Manifest)),

        // Python lockfiles
        "poetry.lock" => Some((Ecosystem::Python, FileType::Lockfile)),
        "uv.lock" => Some((Ecosystem::Python, FileType::Lockfile)),
        "Pipfile.lock" => Some((Ecosystem::Python, FileType::Lockfile)),

        // Rust manifest files
        "Cargo.toml" => Some((Ecosystem::Rust, FileType::Manifest)),

        // Rust lockfiles
        "Cargo.lock" => Some((Ecosystem::Rust, FileType::Lockfile)),

        _ => None,
    }
}
