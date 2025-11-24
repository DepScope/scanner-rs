# Design Document

## Overview

This design transforms the Scanner from a Node.js-focused tool into a comprehensive multi-language dependency analyzer supporting Python, Node.js/TypeScript, and Rust ecosystems. The architecture emphasizes modularity, extensibility, and performance through a parser-based design with parallel processing.

The system will maintain the existing high-performance characteristics (parallel filesystem traversal, configurable thread pools) while adding structured parsing modules for each file format. Each parser will be independently testable and follow a common interface pattern.

## Architecture

### High-Level Structure

```
scanner/
├── src/
│   ├── main.rs                 # CLI entry point, orchestration
│   ├── lib.rs                  # Public library interface
│   ├── indexer/
│   │   ├── mod.rs              # Filesystem indexing and discovery
│   │   └── file_types.rs       # File type identification
│   ├── parsers/
│   │   ├── mod.rs              # Parser trait and registry
│   │   ├── manifest/
│   │   │   ├── mod.rs
│   │   │   ├── package_json.rs
│   │   │   ├── pyproject_toml.rs
│   │   │   ├── requirements_txt.rs
│   │   │   └── cargo_toml.rs
│   │   └── lockfile/
│   │       ├── mod.rs
│   │       ├── yarn_lock.rs
│   │       ├── package_lock_json.rs
│   │       ├── pnpm_lock_yaml.rs
│   │       ├── poetry_lock.rs
│   │       ├── uv_lock.rs
│   │       └── cargo_lock.rs
│   ├── models/
│   │   ├── mod.rs
│   │   ├── dependency.rs       # Core dependency data structures
│   │   └── scan_result.rs      # Scan result aggregation
│   ├── version/
│   │   ├── mod.rs
│   │   ├── node_semver.rs      # Node.js version handling
│   │   ├── rust_semver.rs      # Rust version handling
│   │   └── python_pep440.rs    # Python version handling
│   └── output/
│       ├── mod.rs
│       └── csv_writer.rs       # CSV output generation
└── tests/
    ├── fixtures/               # Test data files
    │   ├── node/
    │   ├── python/
    │   └── rust/
    └── integration/
        └── end_to_end.rs
```

### Component Responsibilities

**Indexer Module**: Fast filesystem traversal to identify relevant package files
- Parallel directory walking with `walkdir` and `rayon`
- File type classification (manifest vs lockfile, ecosystem detection)
- Exclusion rules (node_modules, target, .nx, etc.)

**Parser Module**: Structured parsing of each file format
- Common `Parser` trait defining `parse()` method
- Separate implementations for each file format
- Returns standardized `DependencyRecord` structures

**Models Module**: Core data structures
- `DependencyRecord`: name, version, source_file, dep_type, ecosystem
- `ManifestDependency`: declared dependencies with version ranges
- `LockfileDependency`: resolved dependencies with exact versions
- `ScanResult`: aggregated results across all files

**Version Module**: Ecosystem-specific version handling
- Node.js: `node-semver` crate for npm-style ranges
- Rust: `semver` crate for Cargo-style requirements
- Python: Custom PEP 440 implementation or `pep440_rs` crate

**Output Module**: Result formatting and export
- CSV generation with enhanced columns
- Future: JSON, YAML output formats

## Components and Interfaces

### Parser Trait

```rust
pub trait Parser {
    /// Parse a file and extract dependency information
    fn parse(&self, content: &str, file_path: &Path) -> Result<Vec<DependencyRecord>, ParseError>;
    
    /// Get the ecosystem this parser handles
    fn ecosystem(&self) -> Ecosystem;
    
    /// Get the file type (manifest or lockfile)
    fn file_type(&self) -> FileType;
}

pub enum Ecosystem {
    Node,
    Python,
    Rust,
}

pub enum FileType {
    Manifest,
    Lockfile,
}
```

### Dependency Record Model

```rust
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

pub enum DependencyType {
    Runtime,
    Development,
    Peer,
    Optional,
    Build,
}
```

### Parser Registry

```rust
pub struct ParserRegistry {
    parsers: HashMap<String, Box<dyn Parser>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: HashMap::new(),
        };
        
        // Register all parsers
        registry.register("package.json", Box::new(PackageJsonParser));
        registry.register("yarn.lock", Box::new(YarnLockParser));
        // ... etc
        
        registry
    }
    
    pub fn get_parser(&self, filename: &str) -> Option<&dyn Parser> {
        self.parsers.get(filename).map(|b| b.as_ref())
    }
}
```

## Data Models

### File Discovery Result

```rust
pub struct DiscoveredFile {
    /// Absolute path to the file
    pub path: PathBuf,
    
    /// File name (e.g., "package.json")
    pub filename: String,
    
    /// Parent directory path
    pub directory: PathBuf,
    
    /// Detected ecosystem
    pub ecosystem: Ecosystem,
    
    /// File type classification
    pub file_type: FileType,
}
```

### Scan Configuration

```rust
pub struct ScanConfig {
    /// Root directory to scan
    pub root_dir: PathBuf,
    
    /// Number of parallel threads
    pub num_threads: usize,
    
    /// Whether to scan recursively
    pub recursive: bool,
    
    /// Directories to exclude
    pub exclude_dirs: Vec<String>,
    
    /// Ecosystems to scan (None = all)
    pub ecosystems: Option<Vec<Ecosystem>>,
    
    /// Verbose logging
    pub verbose: bool,
}
```

## Parser Implementation Details

### Node.js Parsers

**package.json** (Manifest)
- Crate: `serde_json`
- Extract: `dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`
- Version format: npm semver ranges (^, ~, >=, etc.)

**yarn.lock** (Lockfile)
- Crate: `yarn-lock-parser` (v0.11.0)
- Extract: Resolved versions for each package entry
- Handles both Yarn v1 and v2 formats

**package-lock.json** (Lockfile)
- Crate: `serde_json` with custom structs
- Extract: Versions from `dependencies` and `packages` sections
- Handles v1, v2, and v3 formats

**pnpm-lock.yaml** (Lockfile)
- Crate: `serde_yaml` with custom structs
- Extract: Package versions from YAML structure
- Parse package keys like `/package/1.2.3`

### Python Parsers

**pyproject.toml** (Manifest)
- Crate: `pyproject-toml` (v0.10.1)
- Extract: `dependencies`, `tool.poetry.dependencies`, `tool.poetry.dev-dependencies`
- Version format: PEP 440 specifiers

**requirements.txt** (Manifest)
- Crate: `requirements` (v0.1.3) or custom parser
- Extract: Package names and version specifiers
- Handle extras, URLs, and comments

**poetry.lock** (Lockfile)
- Crate: `toml` with custom structs
- Extract: Exact versions from `[[package]]` sections
- Parse TOML structure

**uv.lock** (Lockfile)
- Crate: `toml` with custom structs
- Extract: Resolved versions from TOML format
- Similar structure to poetry.lock

### Rust Parsers

**Cargo.toml** (Manifest)
- Crate: `cargo_toml` (v0.21.3)
- Extract: `dependencies`, `dev-dependencies`, `build-dependencies`
- Version format: Cargo version requirements

**Cargo.lock** (Lockfile)
- Crate: `cargo_lock` (v11.0.0)
- Extract: Exact resolved versions
- Supports v1-v4 lockfile formats

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Parse error in {file}: {message}")]
    Parse {
        file: PathBuf,
        message: String,
    },
    
    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
    
    #[error("Version parse error: {0}")]
    VersionParse(String),
}

pub type Result<T> = std::result::Result<T, ScanError>;
```

### Error Recovery Strategy

- **File Read Errors**: Log warning, skip file, continue scanning
- **Parse Errors**: Log error with file path, skip file, continue scanning
- **Version Parse Errors**: Log warning, record raw version string, continue
- **Fatal Errors**: Only for CLI argument errors or output file write failures

## Testing Strategy

### Unit Tests

Each parser module will have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
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
        assert!(result.iter().any(|d| d.name == "react" && d.version == "^18.2.0"));
    }
    
    #[test]
    fn test_parse_malformed_json() {
        let content = "{ invalid json }";
        let parser = PackageJsonParser;
        let result = parser.parse(content, Path::new("package.json"));
        
        assert!(result.is_err());
    }
}
```

### Test Fixtures

Create realistic test files in `tests/fixtures/`:
- `node/package.json` - Real-world package.json with various dependency types
- `node/yarn.lock` - Yarn lockfile with multiple packages
- `python/pyproject.toml` - Poetry project with dependencies
- `python/poetry.lock` - Poetry lockfile
- `rust/Cargo.toml` - Cargo manifest with features
- `rust/Cargo.lock` - Cargo lockfile

### Integration Tests

End-to-end tests that:
1. Create temporary directory structure with test fixtures
2. Run full scan
3. Verify all expected dependencies are found
4. Verify output format correctness

## Performance Considerations

### Parallel Processing

- **File Discovery**: Parallel directory traversal with `rayon::par_iter()`
- **File Parsing**: Parallel parsing of discovered files
- **Thread Pool**: Configurable via CLI (`--jobs` flag)
- **Default Threads**: `num_cpus::get()`

### Memory Optimization

- **Streaming Parsing**: Use streaming JSON/YAML parsers where possible
- **Lazy Loading**: Only parse files when needed
- **Result Accumulation**: Use `Mutex<Vec<>>` for thread-safe collection
- **No Preloading**: Remove the preload pattern from current implementation

### Caching Strategy

- **File Content**: Read once, parse once
- **Parser Registry**: Singleton pattern, created once at startup
- **Version Parsing**: Cache parsed version objects if needed

## Migration from Current Implementation

### Phase 1: Module Structure
1. Create new directory structure
2. Move existing code into appropriate modules
3. Extract common types into models module

### Phase 2: Parser Abstraction
1. Implement Parser trait
2. Refactor existing npm parsers to implement trait
3. Create ParserRegistry

### Phase 3: Add Python Support
1. Implement pyproject.toml parser
2. Implement requirements.txt parser
3. Implement poetry.lock parser
4. Implement uv.lock parser

### Phase 4: Add Rust Support
1. Implement Cargo.toml parser
2. Implement Cargo.lock parser

### Phase 5: Enhanced Output
1. Update DependencyRecord model
2. Modify CSV output to include new fields
3. Add ecosystem and file_type columns

### Backward Compatibility

- Maintain existing CLI flags (`--dir`, `--jobs`, `--verbose`, etc.)
- Keep CSV output format compatible (add new columns at end)
- Preserve `packages.txt` input format for now (deprecate later)

## Dependencies

### New Crates to Add

```toml
[dependencies]
# Existing
clap = { version = "4.0", features = ["derive"] }
rayon = "1.7"
walkdir = "2.4"
csv = "1.3"
num_cpus = "1.16"
serde_json = "1.0"
regex = "1.10"

# New - Core
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
anyhow = "1.0"

# New - Node.js
node-semver = "2.1"
yarn-lock-parser = "0.11"

# New - Python
pyproject-toml = "0.10"
pep440_rs = "0.6"
toml = "0.8"

# New - Rust
cargo_toml = "0.21"
cargo_lock = "11.0"

# New - YAML
serde_yaml = "0.9"
```

## Future Enhancements

### Java Support
- Maven: `pom.xml` (manifest), `pom.xml.lock` (lockfile)
- Gradle: `build.gradle` / `build.gradle.kts` (manifest), `gradle.lockfile` (lockfile)

### Additional Output Formats
- JSON output for programmatic consumption
- YAML output for human readability
- HTML report with visualization

### Dependency Graph
- Build dependency tree showing relationships
- Identify transitive dependencies
- Detect circular dependencies

### Version Analysis
- Identify outdated packages
- Check for security vulnerabilities (integrate with advisory databases)
- Suggest version updates

### Configuration File
- Support `.scannerrc` or `scanner.toml` for project-specific settings
- Define custom exclusion patterns
- Configure output format preferences
