//! Core dependency data structures

use std::path::PathBuf;
use serde::{Deserialize, Serialize};

/// A dependency record representing a package dependency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyRecord {
    /// Package name
    pub name: String,
    
    /// Version specification (range for manifests, exact for lockfiles)
    pub version: String,
    
    /// Source file path
    pub source_file: PathBuf,
    
    /// Dependency type (dependencies, devDependencies, build-dependencies, etc.)
    pub dep_type: DependencyType,
    
    /// Ecosystem
    pub ecosystem: Ecosystem,
    
    /// Whether this is from a manifest or lockfile
    pub file_type: FileType,
}

/// Type of dependency
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum DependencyType {
    /// Runtime/production dependency
    Runtime,
    /// Development dependency
    Development,
    /// Peer dependency
    Peer,
    /// Optional dependency
    Optional,
    /// Build dependency
    Build,
}

impl std::fmt::Display for DependencyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DependencyType::Runtime => write!(f, "runtime"),
            DependencyType::Development => write!(f, "development"),
            DependencyType::Peer => write!(f, "peer"),
            DependencyType::Optional => write!(f, "optional"),
            DependencyType::Build => write!(f, "build"),
        }
    }
}

/// Package ecosystem
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    /// Node.js/npm/yarn/pnpm
    Node,
    /// Python/pip/poetry/uv
    Python,
    /// Rust/Cargo
    Rust,
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Node => write!(f, "node"),
            Ecosystem::Python => write!(f, "python"),
            Ecosystem::Rust => write!(f, "rust"),
        }
    }
}

/// File type classification
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FileType {
    /// Manifest file (declared dependencies)
    Manifest,
    /// Lockfile (resolved/installed versions)
    Lockfile,
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Manifest => write!(f, "manifest"),
            FileType::Lockfile => write!(f, "lockfile"),
        }
    }
}
