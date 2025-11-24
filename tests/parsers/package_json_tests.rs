use std::path::Path;
use scanner::models::{DependencyType, Ecosystem, FileType};
use scanner::parsers::manifest::PackageJsonParser;
use scanner::parsers::Parser;

#[test]
fn test_parse_package_json_dependencies() {
    let content = r#"{
        "dependencies": {
            "react": "^18.2.0",
            "lodash": "~4.17.21"
        },
        "devDependencies": {
            "typescript": "^5.0.0"
        }
    }"#;
    
    let parser = PackageJsonParser;
    let result = parser.parse(content, Path::new("package.json")).unwrap();
    
    assert_eq!(result.len(), 3);
    
    // Check runtime dependencies
    let react = result.iter().find(|d| d.name == "react").unwrap();
    assert_eq!(react.version, "^18.2.0");
    assert_eq!(react.dep_type, DependencyType::Runtime);
    assert_eq!(react.ecosystem, Ecosystem::Node);
    assert_eq!(react.file_type, FileType::Manifest);
    
    let lodash = result.iter().find(|d| d.name == "lodash").unwrap();
    assert_eq!(lodash.version, "~4.17.21");
    assert_eq!(lodash.dep_type, DependencyType::Runtime);
    
    // Check dev dependencies
    let typescript = result.iter().find(|d| d.name == "typescript").unwrap();
    assert_eq!(typescript.version, "^5.0.0");
    assert_eq!(typescript.dep_type, DependencyType::Development);
}

#[test]
fn test_parse_package_json_all_dependency_types() {
    let content = r#"{
        "dependencies": {
            "react": "^18.2.0"
        },
        "devDependencies": {
            "jest": "^29.5.0"
        },
        "peerDependencies": {
            "react-dom": "^18.2.0"
        },
        "optionalDependencies": {
            "fsevents": "^2.3.2"
        }
    }"#;
    
    let parser = PackageJsonParser;
    let result = parser.parse(content, Path::new("package.json")).unwrap();
    
    assert_eq!(result.len(), 4);
    
    let react = result.iter().find(|d| d.name == "react").unwrap();
    assert_eq!(react.dep_type, DependencyType::Runtime);
    
    let jest = result.iter().find(|d| d.name == "jest").unwrap();
    assert_eq!(jest.dep_type, DependencyType::Development);
    
    let react_dom = result.iter().find(|d| d.name == "react-dom").unwrap();
    assert_eq!(react_dom.dep_type, DependencyType::Peer);
    
    let fsevents = result.iter().find(|d| d.name == "fsevents").unwrap();
    assert_eq!(fsevents.dep_type, DependencyType::Optional);
}

#[test]
fn test_parse_empty_dependencies() {
    let content = r#"{
        "name": "empty-project",
        "version": "1.0.0"
    }"#;
    
    let parser = PackageJsonParser;
    let result = parser.parse(content, Path::new("package.json")).unwrap();
    
    assert_eq!(result.len(), 0);
}

#[test]
fn test_parse_malformed_json() {
    let content = "{ invalid json }";
    
    let parser = PackageJsonParser;
    let result = parser.parse(content, Path::new("package.json"));
    
    assert!(result.is_err());
}

#[test]
fn test_parse_fixture_file() {
    let content = std::fs::read_to_string("tests/fixtures/node/package.json").unwrap();
    
    let parser = PackageJsonParser;
    let result = parser.parse(&content, Path::new("tests/fixtures/node/package.json")).unwrap();
    
    // Should have 3 runtime + 3 dev + 1 peer + 1 optional = 8 total
    assert_eq!(result.len(), 8);
    
    // Verify some specific packages
    assert!(result.iter().any(|d| d.name == "react" && d.dep_type == DependencyType::Runtime));
    assert!(result.iter().any(|d| d.name == "typescript" && d.dep_type == DependencyType::Development));
    assert!(result.iter().any(|d| d.name == "react-dom" && d.dep_type == DependencyType::Peer));
    assert!(result.iter().any(|d| d.name == "fsevents" && d.dep_type == DependencyType::Optional));
}

#[test]
fn test_parser_metadata() {
    let parser = PackageJsonParser;
    
    assert_eq!(parser.ecosystem(), Ecosystem::Node);
    assert_eq!(parser.file_type(), FileType::Manifest);
    assert_eq!(parser.filename(), "package.json");
}
