//! # Scanner Library
//!
//! A multi-language dependency scanner for Python, Node.js, and Rust ecosystems.
//!
//! This library provides functionality to discover, parse, and analyze package dependencies
//! across different package management systems.

pub mod analyzer;
pub mod indexer;
pub mod models;
pub mod output;
pub mod parsers;
pub mod version;

// Re-export commonly used types
pub use models::{DependencyRecord, DependencyType, Ecosystem, FileType, ScanResult};
pub use parsers::{Parser, ParserRegistry};

/// Result type for scanner operations
pub type Result<T> = std::result::Result<T, models::ScanError>;
