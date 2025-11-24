//! Manifest file parsers (declared dependencies)

pub mod cargo_toml;
pub mod package_json;
pub mod pyproject_toml;
pub mod requirements_txt;

pub use cargo_toml::CargoTomlParser;
pub use package_json::PackageJsonParser;
pub use pyproject_toml::PyprojectTomlParser;
pub use requirements_txt::RequirementsTxtParser;
