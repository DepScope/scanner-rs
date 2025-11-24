use scanner::models::{DependencyType, Ecosystem, FileType};
use scanner::parsers::manifest::{PyprojectTomlParser, RequirementsTxtParser};
use scanner::parsers::Parser;
use std::path::Path;

#[test]
fn test_parse_pyproject_toml_pep621() {
    let content = r#"
[project]
name = "test"
version = "1.0.0"
dependencies = [
    "requests>=2.28.0",
    "numpy==1.24.0"
]
"#;

    let parser = PyprojectTomlParser;
    let result = parser.parse(content, Path::new("pyproject.toml")).unwrap();

    assert_eq!(result.len(), 2);

    let requests = result.iter().find(|d| d.name == "requests");
    assert!(requests.is_some());
    let requests = requests.unwrap();
    assert_eq!(requests.version, ">=2.28.0");
    assert_eq!(requests.dep_type, DependencyType::Runtime);
    assert_eq!(requests.ecosystem, Ecosystem::Python);

    let numpy = result.iter().find(|d| d.name == "numpy");
    assert!(numpy.is_some());
    assert_eq!(numpy.unwrap().version, "==1.24.0");
}

#[test]
fn test_parse_pyproject_toml_poetry() {
    let content = r#"
[tool.poetry]
name = "test"
version = "1.0.0"

[tool.poetry.dependencies]
python = "^3.9"
django = "^4.2.0"
requests = {version = "^2.28.0"}

[tool.poetry.dev-dependencies]
pytest = "^7.4.0"
"#;

    let parser = PyprojectTomlParser;
    let result = parser.parse(content, Path::new("pyproject.toml")).unwrap();

    // Should have 2 runtime (django, requests) + 1 dev (pytest) = 3
    // python is skipped
    assert_eq!(result.len(), 3);

    let django = result.iter().find(|d| d.name == "django");
    assert!(django.is_some());
    assert_eq!(django.unwrap().version, "^4.2.0");

    let pytest = result.iter().find(|d| d.name == "pytest");
    assert!(pytest.is_some());
    let pytest = pytest.unwrap();
    assert_eq!(pytest.version, "^7.4.0");
    assert_eq!(pytest.dep_type, DependencyType::Development);
}

#[test]
fn test_parse_pyproject_toml_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/python/pyproject.toml").unwrap();

    let parser = PyprojectTomlParser;
    let result = parser
        .parse(&content, Path::new("tests/fixtures/python/pyproject.toml"))
        .unwrap();

    // Should have PEP 621 deps + Poetry deps (excluding python)
    assert!(result.len() >= 5);

    // Check PEP 621 dependencies
    assert!(result
        .iter()
        .any(|d| d.name == "requests" && d.version == ">=2.28.0"));
    assert!(result
        .iter()
        .any(|d| d.name == "numpy" && d.version == "==1.24.0"));

    // Check Poetry dependencies
    assert!(result
        .iter()
        .any(|d| d.name == "django" && d.dep_type == DependencyType::Runtime));
    assert!(result
        .iter()
        .any(|d| d.name == "pytest" && d.dep_type == DependencyType::Development));
}

#[test]
fn test_parse_requirements_txt() {
    let content = r#"
# Core dependencies
requests>=2.28.0
numpy==1.24.0

# Optional
flask>=3.0.0
"#;

    let parser = RequirementsTxtParser;
    let result = parser
        .parse(content, Path::new("requirements.txt"))
        .unwrap();

    assert_eq!(result.len(), 3);

    let requests = result.iter().find(|d| d.name == "requests");
    assert!(requests.is_some());
    assert_eq!(requests.unwrap().version, ">=2.28.0");

    let numpy = result.iter().find(|d| d.name == "numpy");
    assert!(numpy.is_some());
    assert_eq!(numpy.unwrap().version, "==1.24.0");
}

#[test]
fn test_parse_requirements_txt_with_extras() {
    let content = "celery[redis]>=5.3.0\nclick";

    let parser = RequirementsTxtParser;
    let result = parser
        .parse(content, Path::new("requirements.txt"))
        .unwrap();

    assert_eq!(result.len(), 2);

    let celery = result.iter().find(|d| d.name == "celery");
    assert!(celery.is_some());
    assert_eq!(celery.unwrap().version, ">=5.3.0");

    let click = result.iter().find(|d| d.name == "click");
    assert!(click.is_some());
    assert_eq!(click.unwrap().version, "*");
}

#[test]
fn test_parse_requirements_txt_fixture() {
    let content = std::fs::read_to_string("tests/fixtures/python/requirements.txt").unwrap();

    let parser = RequirementsTxtParser;
    let result = parser
        .parse(
            &content,
            Path::new("tests/fixtures/python/requirements.txt"),
        )
        .unwrap();

    assert!(result.len() >= 5);
    assert!(result
        .iter()
        .any(|d| d.name == "requests" && d.version == ">=2.28.0"));
    assert!(result
        .iter()
        .any(|d| d.name == "numpy" && d.version == "==1.24.0"));
    assert!(result
        .iter()
        .any(|d| d.name == "celery" && d.version == ">=5.3.0"));
    assert!(result.iter().any(|d| d.name == "click" && d.version == "*"));
}

#[test]
fn test_pyproject_toml_parser_metadata() {
    let parser = PyprojectTomlParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Python);
    assert_eq!(parser.file_type(), FileType::Manifest);
    assert_eq!(parser.filename(), "pyproject.toml");
}

#[test]
fn test_requirements_txt_parser_metadata() {
    let parser = RequirementsTxtParser;
    assert_eq!(parser.ecosystem(), Ecosystem::Python);
    assert_eq!(parser.file_type(), FileType::Manifest);
    assert_eq!(parser.filename(), "requirements.txt");
}
