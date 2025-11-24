use std::path::Path;
use scanner::models::{DependencyType, Ecosystem, FileType};
use scanner::parsers::lockfile::{YarnLockParser, PackageLockJsonParser, PnpmLockParser};
use scanner::parsers::Parser;

#[test]
fn test_parse_yarn_lock() {
    let content = r#"
# yarn lockfile v1

lodash@^4.17.21:
  version "4.17.21"
  resolved "https://registry.yarnpkg.com/lodash/-/lodash-4.17.21.tgz"

react@^18.2.0:
  version "18.2.0"
  resolved "https://registry.yarnpkg.com/react/-/react-18.2.0.tgz"
"#;
    
    let parser = YarnLockParser;
    let result = parser.parse(content, Path::new("yarn.lock")).unwrap();
    
    assert!(result.len() >= 2);
    
    let lodash = result.iter().find(|d| d.name == "lodash");
    assert!(lodash.is_some());
    let lodash = lodash.unwrap();
    assert_eq!(lodash.version, "4.17.21");
    assert_eq!(lodash.ecosystem, Ecosystem::Node);
    assert_eq!(lodash.file_type, FileType::Lockfile);
    
    let react = result.iter().find(|d| d.name == "react");
    assert!(react.is_some());
    let react = react.unwrap();
    assert_eq!(react.version, "18.2.0");
}

#[test]
fn test_parse_yarn_lock_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/node/yarn.lock").unwrap();
    
    let parser = YarnLockParser;
    let result = parser.parse(&content, Path::new("tests/fixtures/node/yarn.lock")).unwrap();
    
    assert!(result.len() >= 3);
    assert!(result.iter().any(|d| d.name == "react" && d.version == "18.2.0"));
    assert!(result.iter().any(|d| d.name == "lodash" && d.version == "4.17.21"));
}

#[test]
fn test_parse_package_lock_json() {
    let content = r#"{
  "name": "test",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {
    "": {
      "name": "test",
      "version": "1.0.0"
    },
    "node_modules/react": {
      "version": "18.2.0"
    },
    "node_modules/lodash": {
      "version": "4.17.21"
    }
  }
}"#;
    
    let parser = PackageLockJsonParser;
    let result = parser.parse(content, Path::new("package-lock.json")).unwrap();
    
    assert_eq!(result.len(), 2);
    
    let react = result.iter().find(|d| d.name == "react");
    assert!(react.is_some());
    assert_eq!(react.unwrap().version, "18.2.0");
    
    let lodash = result.iter().find(|d| d.name == "lodash");
    assert!(lodash.is_some());
    assert_eq!(lodash.unwrap().version, "4.17.21");
}

#[test]
fn test_parse_package_lock_json_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/node/package-lock.json").unwrap();
    
    let parser = PackageLockJsonParser;
    let result = parser.parse(&content, Path::new("tests/fixtures/node/package-lock.json")).unwrap();
    
    assert!(result.len() >= 3);
    assert!(result.iter().any(|d| d.name == "react" && d.version == "18.2.0"));
    assert!(result.iter().any(|d| d.name == "lodash" && d.version == "4.17.21"));
    assert!(result.iter().any(|d| d.name == "axios" && d.version == "1.4.0"));
}

#[test]
fn test_parse_pnpm_lock_yaml() {
    let content = r#"
lockfileVersion: '6.0'

packages:
  /lodash/4.17.21:
    resolution: {integrity: sha512-xyz...}
  
  /react/18.2.0:
    resolution: {integrity: sha512-abc...}
"#;
    
    let parser = PnpmLockParser;
    let result = parser.parse(content, Path::new("pnpm-lock.yaml")).unwrap();
    
    assert!(result.len() >= 2);
    
    let lodash = result.iter().find(|d| d.name == "lodash" && d.version == "4.17.21");
    assert!(lodash.is_some());
    
    let react = result.iter().find(|d| d.name == "react" && d.version == "18.2.0");
    assert!(react.is_some());
}

#[test]
fn test_parse_pnpm_lock_yaml_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/node/pnpm-lock.yaml").unwrap();
    
    let parser = PnpmLockParser;
    let result = parser.parse(&content, Path::new("tests/fixtures/node/pnpm-lock.yaml")).unwrap();
    
    assert!(result.len() >= 2);
    assert!(result.iter().any(|d| d.name == "react" && d.version == "18.2.0"));
    assert!(result.iter().any(|d| d.name == "lodash" && d.version == "4.17.21"));
}

#[test]
fn test_yarn_lock_parser_metadata() {
    let parser = YarnLockParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Node);
    assert_eq!(parser.file_type(), FileType::Lockfile);
    assert_eq!(parser.filename(), "yarn.lock");
}

#[test]
fn test_package_lock_parser_metadata() {
    let parser = PackageLockJsonParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Node);
    assert_eq!(parser.file_type(), FileType::Lockfile);
    assert_eq!(parser.filename(), "package-lock.json");
}

#[test]
fn test_pnpm_lock_parser_metadata() {
    let parser = PnpmLockParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Node);
    assert_eq!(parser.file_type(), FileType::Lockfile);
    assert_eq!(parser.filename(), "pnpm-lock.yaml");
}
