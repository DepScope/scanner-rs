use scanner::models::{Ecosystem, FileType};
use scanner::parsers::lockfile::{PoetryLockParser, UvLockParser};
use scanner::parsers::Parser;
use std::path::Path;

#[test]
fn test_parse_poetry_lock() {
    let content = r#"
[[package]]
name = "django"
version = "4.2.3"
description = "A high-level Python web framework"

[[package]]
name = "requests"
version = "2.31.0"
description = "Python HTTP for Humans."
"#;

    let parser = PoetryLockParser;
    let result = parser.parse(content, Path::new("poetry.lock")).unwrap();

    assert_eq!(result.len(), 2);

    let django = result.iter().find(|d| d.name == "django");
    assert!(django.is_some());
    let django = django.unwrap();
    assert_eq!(django.version, "4.2.3");
    assert_eq!(django.ecosystem, Ecosystem::Python);
    assert_eq!(django.file_type, FileType::Lockfile);

    let requests = result.iter().find(|d| d.name == "requests");
    assert!(requests.is_some());
    assert_eq!(requests.unwrap().version, "2.31.0");
}

#[test]
fn test_parse_poetry_lock_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/python/poetry.lock").unwrap();

    let parser = PoetryLockParser;
    let result = parser
        .parse(&content, Path::new("tests/fixtures/python/poetry.lock"))
        .unwrap();

    assert_eq!(result.len(), 3);
    assert!(result
        .iter()
        .any(|d| d.name == "django" && d.version == "4.2.3"));
    assert!(result
        .iter()
        .any(|d| d.name == "requests" && d.version == "2.31.0"));
    assert!(result
        .iter()
        .any(|d| d.name == "pytest" && d.version == "7.4.0"));
}

#[test]
fn test_parse_uv_lock() {
    let content = r#"
version = 1

[[package]]
name = "flask"
version = "3.0.0"
source = { registry = "https://pypi.org/simple" }

[[package]]
name = "click"
version = "8.1.7"
source = { registry = "https://pypi.org/simple" }
"#;

    let parser = UvLockParser;
    let result = parser.parse(content, Path::new("uv.lock")).unwrap();

    assert_eq!(result.len(), 2);

    let flask = result.iter().find(|d| d.name == "flask");
    assert!(flask.is_some());
    assert_eq!(flask.unwrap().version, "3.0.0");

    let click = result.iter().find(|d| d.name == "click");
    assert!(click.is_some());
    assert_eq!(click.unwrap().version, "8.1.7");
}

#[test]
fn test_parse_uv_lock_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/python/uv.lock").unwrap();

    let parser = UvLockParser;
    let result = parser
        .parse(&content, Path::new("tests/fixtures/python/uv.lock"))
        .unwrap();

    assert_eq!(result.len(), 3);
    assert!(result
        .iter()
        .any(|d| d.name == "flask" && d.version == "3.0.0"));
    assert!(result
        .iter()
        .any(|d| d.name == "click" && d.version == "8.1.7"));
    assert!(result
        .iter()
        .any(|d| d.name == "werkzeug" && d.version == "3.0.1"));
}

#[test]
fn test_poetry_lock_parser_metadata() {
    let parser = PoetryLockParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Python);
    assert_eq!(parser.file_type(), FileType::Lockfile);
    assert_eq!(parser.filename(), "poetry.lock");
}

#[test]
fn test_uv_lock_parser_metadata() {
    let parser = UvLockParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Python);
    assert_eq!(parser.file_type(), FileType::Lockfile);
    assert_eq!(parser.filename(), "uv.lock");
}
