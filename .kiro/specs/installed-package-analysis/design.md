# Design Document

## Overview

This design extends the Scanner to detect physically installed packages in filesystem locations (node_modules, site-packages, virtual environments) and establish dependency relationships with a three-tier classification system (HAS/SHOULD/CAN).
This enables supply chain security analysis by identifying which systems have vulnerable packages actually installed versus merely declared.

The design builds on the existing multi-language scanner architecture, adding new detection modules for installed packages, dependency tree construction, and enhanced output formats. The system maintains high performance through parallel processing while adding sophisticated relationship tracking.

## Architecture

### Extended Module Structure

```text
scanner/
├── src/
│   ├── main.rs                      # CLI entry point, orchestration
│   ├── lib.rs                       # Public library interface
│   ├── indexer/
│   │   ├── mod.rs                   # Filesystem indexing and discovery
│   │   ├── file_types.rs            # File type identification
│   │   └── install_dirs.rs          # NEW: Installation directory detection
│   ├── parsers/
│   │   ├── mod.rs                   # Parser trait and registry
│   │   ├── manifest/                # Existing manifest parsers
│   │   ├── lockfile/                # Existing lockfile parsers
│   │   └── installed/               # NEW: Installed package parsers
│   │       ├── mod.rs
│   │       ├── node_modules.rs      # Parse node_modules packages
│   │       ├── site_packages.rs     # Parse Python site-packages
│   │       └── metadata.rs          # Parse Python .dist-info/.egg-info
│   ├── models/
│   │   ├── mod.rs
│   │   ├── dependency.rs            # Core dependency data structures
│   │   ├── scan_result.rs           # Scan result aggregation
│   │   ├── classification.rs        # NEW: HAS/SHOULD/CAN classification
│   │   ├── dependency_tree.rs       # NEW: Dependency relationship tree
│   │   └── application.rs           # NEW: Application root tracking
│   ├── analyzer/                    # NEW: Analysis and relationship building
│   │   ├── mod.rs
│   │   ├── classifier.rs            # Assign HAS/SHOULD/CAN classifications
│   │   ├── tree_builder.rs          # Build dependency trees
│   │   ├── version_matcher.rs       # Match and compare versions
│   │   └── app_linker.rs            # Link packages to application roots
│   ├── version/                     # Existing version handling
│   └── output/
│       ├── mod.rs
│       ├── csv_writer.rs            # Enhanced CSV output
│       └── json_writer.rs           # NEW: JSON tree output
└── tests/
    ├── fixtures/
    │   ├── node/
    │   │   └── node_modules/        # NEW: Sample installed packages
    │   ├── python/
    │   │   └── site-packages/       # NEW: Sample installed packages
    │   └── rust/
    └── integration/
        └── installed_analysis.rs    # NEW: Integration tests
```text

### New Component Responsibilities

**Install Directory Detection**: Identify package installation locations

- Detect node_modules directories and parse package.json files within
- Detect site-packages/dist-packages and parse .dist-info/.egg-info metadata
- Identify virtual environments (venv, .venv, pyenv)
- Track installation paths and link to parent directories

**Installed Package Parsers**: Extract metadata from installed packages

- Node.js: Read package.json from each node_modules subdirectory
- Python: Parse METADATA files from .dist-info directories
- Python: Parse PKG-INFO files from .egg-info directories
- Extract name, version, and dependency information

**Analyzer Module**: Build relationships and classifications

- Classify each package as HAS, SHOULD, and/or CAN
- Link installed packages to their declaring application root
- Build dependency trees showing parent-child relationships
- Detect version mismatches between classifications
- Filter results based on vulnerability lists

**Enhanced Output**: Multiple output formats with rich data

- CSV: Extended columns for classifications, paths, relationships
- JSON: Hierarchical tree structure with nested dependencies
- Support filtering and sorting by classification priority

## Components and Interfaces

### Classification Model

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Classification {
    Has,      // Physically installed
    Should,   // In lock file
    Can,      // In manifest
}

#[derive(Debug, Clone)]
pub struct ClassifiedDependency {
    /// Package name
    pub name: String,

    /// Classifications with their associated versions
    pub classifications: HashMap<Classification, String>,

    /// Ecosystem
    pub ecosystem: Ecosystem,

    /// Application root (nearest manifest file)
    pub application_root: Option<PathBuf>,

    /// Application name (from manifest)
    pub application_name: Option<String>,

    /// Installed package path (for HAS classification)
    pub installed_path: Option<PathBuf>,

    /// Source files for each classification
    pub source_files: HashMap<Classification, PathBuf>,

    /// Version mismatch flags
    pub has_version_mismatch: bool,
    pub has_constraint_violation: bool,

    /// Parent package (for dependency tree)
    pub parent_package: Option<String>,

    /// Direct dependencies
    pub dependencies: Vec<String>,
}
```text

### Application Root Model

```rust
#[derive(Debug, Clone)]
pub struct Application {
    /// Application name (from package.json, pyproject.toml, etc.)
    pub name: String,

    /// Absolute path to application root directory
    pub root_path: PathBuf,

    /// Manifest file path
    pub manifest_path: PathBuf,

    /// Ecosystem
    pub ecosystem: Ecosystem,

    /// All dependencies associated with this application
    pub dependencies: Vec<ClassifiedDependency>,
}
```text

### Dependency Tree Model

```rust
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Package name
    pub name: String,

    /// Package version
    pub version: String,

    /// Classification
    pub classification: Classification,

    /// Direct dependencies (children in tree)
    pub dependencies: Vec<DependencyNode>,

    /// Whether this is a direct or transitive dependency
    pub is_direct: bool,
}

#[derive(Debug)]
pub struct DependencyTree {
    /// Root application
    pub application: Application,

    /// Top-level dependency nodes
    pub roots: Vec<DependencyNode>,
}
```text

### Installed Package Parser Trait

```rust
pub trait InstalledPackageParser {
    /// Parse installed package metadata
    fn parse_installed(&self, install_dir: &Path) -> Result<Vec<InstalledPackage>, ParseError>;

    /// Get the ecosystem this parser handles
    fn ecosystem(&self) -> Ecosystem;
}

#[derive(Debug, Clone)]
pub struct InstalledPackage {
    /// Package name
    pub name: String,

    /// Installed version
    pub version: String,

    /// Installation path
    pub path: PathBuf,

    /// Ecosystem
    pub ecosystem: Ecosystem,

    /// Direct dependencies declared by this package
    pub dependencies: Vec<DependencySpec>,
}

#[derive(Debug, Clone)]
pub struct DependencySpec {
    /// Dependency name
    pub name: String,

    /// Version constraint
    pub version_constraint: String,
}
```text

## Installation Directory Detection

### Node.js Detection

**node_modules Discovery**:

- Scan for directories named `node_modules`
- For each subdirectory in node_modules, look for `package.json`
- Parse package.json to extract name, version, and dependencies
- Handle scoped packages (@org/package)
- Support nested node_modules (transitive dependencies)

**Example Structure**:

```text
project/
├── package.json              # CAN classification
├── package-lock.json         # SHOULD classification
└── node_modules/             # HAS classification
    ├── react/
    │   ├── package.json      # name: "react", version: "18.2.0"
    │   └── node_modules/     # Nested transitive deps
    └── lodash/
        └── package.json      # name: "lodash", version: "4.17.21"
```text

### Python Detection

**site-packages Discovery**:

- Scan for directories named `site-packages` or `dist-packages`
- Look for `.dist-info` directories (modern format)
- Look for `.egg-info` directories or files (legacy format)
- Parse METADATA or PKG-INFO files

**Virtual Environment Detection**:

- Look for `pyvenv.cfg` file (indicates venv)
- Check for `.venv`, `venv`, `env` directory names
- Scan the venv's site-packages directory

**Metadata Parsing**:

```text
site-packages/
├── requests-2.31.0.dist-info/
│   ├── METADATA              # Contains Name, Version, Requires-Dist
│   └── RECORD
└── urllib3-2.0.7.dist-info/
    └── METADATA
```text

METADATA format:

```text
Metadata-Version: 2.1
Name: requests
Version: 2.31.0
Requires-Dist: charset-normalizer (<4,>=2)
Requires-Dist: idna (<4,>=2.5)
Requires-Dist: urllib3 (<3,>=1.21.1)
```text

## Analyzer Module Design

### Classifier Component

**Purpose**: Assign HAS/SHOULD/CAN classifications to each unique package

**Algorithm**:

1. Collect all dependency records from parsers (manifest, lockfile, installed)
2. Group by package name and ecosystem
3. For each package:
   - If found in installed parsers → add HAS classification with installed version
   - If found in lockfile parsers → add SHOULD classification with locked version
   - If found in manifest parsers → add CAN classification with version range
4. Create `ClassifiedDependency` objects with all applicable classifications

**Version Mismatch Detection**:

- Compare HAS version with SHOULD version (exact match expected)
- Verify SHOULD version satisfies CAN version range
- Set flags: `has_version_mismatch`, `has_constraint_violation`

### Application Linker Component

**Purpose**: Link installed packages to their declaring application

**Algorithm**:

1. For each installed package path, traverse parent directories
2. Look for manifest files (package.json, pyproject.toml, Cargo.toml)
3. First manifest found is the application root
4. Parse manifest to extract application name
5. Associate installed package with application

**Example**:

```text
/home/user/projects/myapp/
├── package.json              # Application: "myapp"
└── node_modules/
    └── react/                # Linked to application "myapp"
```text

### Tree Builder Component

**Purpose**: Construct dependency trees showing relationships

**Algorithm**:

1. Start with installed packages (HAS classification)
2. For each installed package, read its dependencies from package.json or METADATA
3. Recursively build tree by looking up each dependency
4. Mark direct vs transitive dependencies
5. Handle circular dependencies (track visited nodes)

**Data Structure**:

```rust
Application: myapp
├── react@18.2.0 (HAS, direct)
│   └── loose-envify@1.4.0 (HAS, transitive)
│       └── js-tokens@4.0.0 (HAS, transitive)
└── lodash@4.17.21 (HAS, direct)
```text

### Version Matcher Component

**Purpose**: Compare versions across classifications

**Functions**:

- `exact_match(v1: &str, v2: &str) -> bool`: Compare exact versions
- `satisfies_range(version: &str, range: &str, ecosystem: Ecosystem) -> bool`: Check if version satisfies range
- `parse_version(version: &str, ecosystem: Ecosystem) -> Result<Version>`: Parse version using ecosystem-specific logic

**Ecosystem-Specific Matching**:

- Node.js: Use `node-semver` crate for range matching
- Python: Use `pep440_rs` for PEP 440 version specifiers
- Rust: Use `semver` crate for Cargo version requirements

## Data Flow

### Full Scan Mode

```text
1. CLI Parsing
   ↓
2. Indexer: Discover all files
   - Manifest files (package.json, pyproject.toml)
   - Lock files (package-lock.json, poetry.lock)
   - Installation directories (node_modules, site-packages)
   ↓
3. Parallel Parsing
   - Manifest parsers → CAN dependencies
   - Lockfile parsers → SHOULD dependencies
   - Installed parsers → HAS dependencies
   ↓
4. Analyzer: Classify
   - Group by package name
   - Assign classifications
   - Detect version mismatches
   ↓
5. Analyzer: Link Applications
   - Find application roots
   - Associate packages with applications
   ↓
6. Analyzer: Build Trees
   - Construct dependency hierarchies
   - Mark direct vs transitive
   ↓
7. Output Generation
   - CSV: Flat table with all data
   - JSON: Hierarchical tree structure
```text

### Installed-Only Scan Mode

```text
1. CLI Parsing (--scan-mode installed-only)
   ↓
2. Indexer: Discover installation directories only
   - node_modules
   - site-packages
   ↓
3. Parallel Parsing
   - Installed parsers → HAS dependencies
   ↓
4. Analyzer: Link Applications
   - Find application roots
   ↓
5. Output Generation
   - Only HAS classification data
```text

### Vulnerability Check Mode

```text
1. CLI Parsing (--vuln-list vulnerabilities.txt)
   ↓
2. Load vulnerability list
   - Parse package@version entries
   ↓
3. Full scan (as above)
   ↓
4. Filter results
   - Match package names and versions
   - Include all classifications for matches
   ↓
5. Sort by priority
   - HAS first (highest priority)
   - SHOULD second
   - CAN last
   ↓
6. Output Generation
   - Only matching packages
```text

## Output Formats

### Enhanced CSV Format

**Columns**:

```text
package_name,ecosystem,application_name,application_root,
has_version,has_path,should_version,should_path,
can_version,can_path,version_mismatch,constraint_violation,
parent_package,is_direct,dependency_depth
```text

**Example**:

```csv
package_name,ecosystem,application_name,application_root,has_version,has_path,should_version,should_path,can_version,can_path,version_mismatch,constraint_violation,parent_package,is_direct,dependency_depth
react,node,myapp,/home/user/myapp,18.2.0,/home/user/myapp/node_modules/react,18.2.0,/home/user/myapp/package-lock.json,^18.0.0,/home/user/myapp/package.json,false,false,,true,0
lodash,node,myapp,/home/user/myapp,4.17.21,/home/user/myapp/node_modules/lodash,4.17.21,/home/user/myapp/package-lock.json,~4.17.0,/home/user/myapp/package.json,false,false,,true,0
loose-envify,node,myapp,/home/user/myapp,1.4.0,/home/user/myapp/node_modules/react/node_modules/loose-envify,1.4.0,/home/user/myapp/package-lock.json,,,false,false,react,false,1
```text

### JSON Tree Format

**Structure**:

```json
{
  "applications": [
    {
      "name": "myapp",
      "root_path": "/home/user/myapp",
      "manifest_path": "/home/user/myapp/package.json",
      "ecosystem": "node",
      "dependencies": [
        {
          "name": "react",
          "classifications": {
            "has": {
              "version": "18.2.0",
              "path": "/home/user/myapp/node_modules/react"
            },
            "should": {
              "version": "18.2.0",
              "path": "/home/user/myapp/package-lock.json"
            },
            "can": {
              "version": "^18.0.0",
              "path": "/home/user/myapp/package.json"
            }
          },
          "version_mismatch": false,
          "constraint_violation": false,
          "is_direct": true,
          "dependencies": [
            {
              "name": "loose-envify",
              "classifications": {
                "has": {
                  "version": "1.4.0",
                  "path": "/home/user/myapp/node_modules/react/node_modules/loose-envify"
                },
                "should": {
                  "version": "1.4.0",
                  "path": "/home/user/myapp/package-lock.json"
                }
              },
              "version_mismatch": false,
              "is_direct": false,
              "dependencies": []
            }
          ]
        }
      ]
    }
  ]
}
```text

## CLI Interface

### New Flags

```bash
# Scan mode
--scan-mode <MODE>           # full (default), installed-only, declared-only

# Output format
--format <FORMAT>            # csv (default), json

# Installation directory handling
--include-install-dirs       # Include node_modules/site-packages in traversal

# Vulnerability checking
--vuln-list <FILE>           # File with package@version entries to check

# Output file
--output <FILE>              # Output file path (default: output.csv or output.json)

# Existing flags (preserved)
--dir <DIR>                  # Root directory to scan
--jobs <N>                   # Number of parallel threads
--verbose                    # Verbose logging
--root-only                  # Only scan root directory
```text

### Example Usage

**Full scan with all classifications**:

```bash
scanner --dir /home/user/projects --format json --output results.json
```text

**Installed packages only**:

```bash
scanner --scan-mode installed-only --dir /home/user/projects
```text

**Vulnerability check**:

```bash
scanner --vuln-list vulnerabilities.txt --dir /home/user/projects --format csv
```text

**Include installation directories in traversal**:

```bash
scanner --include-install-dirs --dir /home/user/projects
```text

## Error Handling

### New Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Failed to link package {package} to application root")]
    ApplicationLinkError { package: String },

    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    #[error("Failed to parse installed package metadata at {path}: {message}")]
    MetadataParseError { path: PathBuf, message: String },

    #[error("Version comparison failed: {message}")]
    VersionComparisonError { message: String },
}
```text

### Error Recovery

- **Missing Application Root**: Log warning, record package without application link
- **Circular Dependencies**: Detect cycle, log warning, break cycle in tree
- **Metadata Parse Errors**: Log error, skip package, continue scanning
- **Version Comparison Errors**: Log warning, skip comparison, record raw versions

## Performance Considerations

### Parallel Processing Strategy

- **Installation Directory Scanning**: Parallel traversal of node_modules subdirectories
- **Metadata Parsing**: Parallel parsing of .dist-info directories
- **Classification**: Single-threaded (requires global view of all packages)
- **Tree Building**: Parallel construction of trees per application

### Memory Optimization

- **Streaming Metadata**: Parse METADATA files line-by-line
- **Lazy Tree Building**: Only build trees when JSON output is requested
- **Deduplication**: Use HashSet to track unique packages before classification

### Caching

- **Application Root Cache**: Cache manifest file locations to avoid repeated traversal
- **Version Parse Cache**: Cache parsed version objects for repeated comparisons
- **Dependency Spec Cache**: Cache parsed dependency specifications

## Testing Strategy

### Unit Tests

**Installed Package Parsers**:

```rust
#[test]
fn test_parse_node_modules_package() {
    let package_json = r#"{"name": "react", "version": "18.2.0", "dependencies": {"loose-envify": "^1.1.0"}}"#;
    let parser = NodeModulesParser;
    let result = parser.parse_installed(Path::new("node_modules/react")).unwrap();

    assert_eq!(result[0].name, "react");
    assert_eq!(result[0].version, "18.2.0");
    assert_eq!(result[0].dependencies.len(), 1);
}

#[test]
fn test_parse_dist_info_metadata() {
    let metadata = "Metadata-Version: 2.1\nName: requests\nVersion: 2.31.0\nRequires-Dist: urllib3 (<3,>=1.21.1)\n";
    let parser = SitePackagesParser;
    let result = parser.parse_metadata(metadata).unwrap();

    assert_eq!(result.name, "requests");
    assert_eq!(result.version, "2.31.0");
    assert_eq!(result.dependencies.len(), 1);
}
```text

**Classifier**:

```rust
#[test]
fn test_classify_all_three() {
    let records = vec![
        DependencyRecord { name: "react".into(), version: "18.2.0".into(), file_type: FileType::Installed, ... },
        DependencyRecord { name: "react".into(), version: "18.2.0".into(), file_type: FileType::Lockfile, ... },
        DependencyRecord { name: "react".into(), version: "^18.0.0".into(), file_type: FileType::Manifest, ... },
    ];

    let classifier = Classifier::new();
    let classified = classifier.classify(records).unwrap();

    assert_eq!(classified.len(), 1);
    assert!(classified[0].classifications.contains_key(&Classification::Has));
    assert!(classified[0].classifications.contains_key(&Classification::Should));
    assert!(classified[0].classifications.contains_key(&Classification::Can));
}
```text

**Version Matcher**:

```rust
#[test]
fn test_version_satisfies_range() {
    let matcher = VersionMatcher::new();
    assert!(matcher.satisfies_range("18.2.0", "^18.0.0", Ecosystem::Node).unwrap());
    assert!(!matcher.satisfies_range("17.0.0", "^18.0.0", Ecosystem::Node).unwrap());
}
```text

### Integration Tests

**End-to-End Installed Analysis**:

1. Create temporary directory with:
   - package.json with dependencies
   - package-lock.json with locked versions
   - node_modules with installed packages
2. Run full scan
3. Verify all three classifications are detected
4. Verify application linking
5. Verify dependency tree structure

### Test Fixtures

**Node.js**:

```text
tests/fixtures/node/
├── package.json
├── package-lock.json
└── node_modules/
    ├── react/
    │   ├── package.json
    │   └── node_modules/
    │       └── loose-envify/
    │           └── package.json
    └── lodash/
        └── package.json
```text

**Python**:

```text
tests/fixtures/python/
├── pyproject.toml
├── poetry.lock
└── .venv/
    └── lib/python3.11/site-packages/
        ├── requests-2.31.0.dist-info/
        │   └── METADATA
        └── urllib3-2.0.7.dist-info/
            └── METADATA
```text

## Dependencies

### New Crates

```toml
[dependencies]
# Existing (from multi-language scanner)
clap = { version = "4.0", features = ["derive"] }
rayon = "1.7"
walkdir = "2.4"
csv = "1.3"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"

# New for installed package analysis
# (Most functionality can be built with existing crates)
```text

No additional crates required - the design leverages existing JSON/TOML parsing capabilities.

## Migration Path

### Phase 1: Installation Directory Detection

1. Add `install_dirs.rs` to indexer module
2. Implement node_modules and site-packages detection
3. Add virtual environment detection

### Phase 2: Installed Package Parsers

1. Create `parsers/installed/` module
2. Implement NodeModulesParser
3. Implement SitePackagesParser with METADATA parsing

### Phase 3: Classification System

1. Create `models/classification.rs`
2. Implement Classifier component
3. Add version mismatch detection

### Phase 4: Application Linking

1. Create `analyzer/app_linker.rs`
2. Implement application root discovery
3. Link installed packages to applications

### Phase 5: Dependency Trees

1. Create `analyzer/tree_builder.rs`
2. Implement tree construction algorithm
3. Handle circular dependencies

### Phase 6: Enhanced Output

1. Extend CSV writer with new columns
2. Implement JSON tree writer
3. Add filtering and sorting

### Phase 7: CLI Integration

1. Add new CLI flags
2. Implement scan mode switching
3. Add vulnerability list support

## Future Enhancements

### Advanced Dependency Analysis

- Detect unused dependencies (declared but not imported)
- Identify duplicate dependencies at different versions
- Suggest dependency consolidation opportunities

### Security Integration

- Integrate with CVE databases (NVD, OSV)
- Automatic vulnerability scanning
- Generate security reports with CVSS scores

### Visualization

- Generate dependency graph visualizations (GraphViz, D3.js)
- Interactive HTML reports
- Dependency size analysis

### Performance Optimization

- Incremental scanning (only scan changed directories)
- Database backend for large codebases
- Distributed scanning for massive monorepos
