//! Core data models for the scanner

pub mod dependency;
pub mod error;
pub mod scan_result;

pub use dependency::{DependencyRecord, DependencyType, Ecosystem, FileType};
pub use error::ScanError;
pub use scan_result::ScanResult;
