//! Core data models for the scanner

pub mod application;
pub mod classification;
pub mod dependency;
pub mod dependency_tree;
pub mod error;
pub mod installed_package;
pub mod scan_result;

pub use application::Application;
pub use classification::{Classification, ClassifiedDependency};
pub use dependency::{DependencyRecord, DependencyType, Ecosystem, FileType};
pub use dependency_tree::{DependencyNode, DependencyTree};
pub use error::ScanError;
pub use installed_package::{DependencySpec, InstalledPackage};
pub use scan_result::ScanResult;
