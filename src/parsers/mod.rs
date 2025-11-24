//! Parser modules for different file formats

use crate::models::{DependencyRecord, Ecosystem, FileType, ScanError};
use std::path::Path;

pub mod installed;
pub mod lockfile;
pub mod manifest;
pub mod registry;

pub use installed::{NodeModulesParser, SitePackagesParser};
pub use registry::ParserRegistry;

/// Parser trait for extracting dependencies from files
pub trait Parser: Send + Sync {
    /// Parse a file and extract dependency information
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ScanError>;

    /// Get the ecosystem this parser handles
    fn ecosystem(&self) -> Ecosystem;

    /// Get the file type (manifest or lockfile)
    fn file_type(&self) -> FileType;

    /// Get the filename this parser handles
    fn filename(&self) -> &str;
}
