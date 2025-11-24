# Installed Package Analysis

This document describes the installed package analysis features added to Scanner.

## Overview

Scanner now detects and analyzes actually installed packages in addition to declared dependencies, providing a comprehensive view of the dependency landscape for supply chain security analysis.

## Three-Tier Classification System

### HAS (Physically Installed)

- Package is found in the filesystem
- **Node.js**: Located in `node_modules/` directories
- **Python**: Located in `site-packages/` or `dist-packages/` directories
- Indicates the package is actually present and can be executed

### SHOULD (Locked Version)

- Package version is specified in a lock file
- **Node.js**: `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`
- **Python**: `poetry.lock`, `uv.lock`
- **Rust**: `Cargo.lock`
- Indicates the intended installation version

### CAN (Declared Dependency)

- Package is declared in a manifest with a version range
- **Node.js**: `package.json`
- **Python**: `pyproject.toml`, `requirements.txt`
- **Rust**: `Cargo.toml`
- Indicates allowed versions

## Features Implemented

### 1. Installation Directory Detection

**Module**: `src/indexer/install_dirs.rs`

Detects package installation locations:

- `node_modules/` directories (Node.js)
- `site-packages/` and `dist-packages/` (Python)
- Virtual environments (`.venv`, `venv`, `env`, detected via `pyvenv.cfg`)

### 2. Installed Package Parsers

#### Node.js Parser

**Module**: `src/parsers/installed/node_modules.rs`

- Reads `package.json` from each subdirectory in `node_modules/`
- Extracts package name, version, and dependencies
- Handles scoped packages (`@org/package`)
- Recursively scans nested `node_modules/` for transitive dependencies

#### Python Parser

**Modules**:

- `src/parsers/installed/metadata.rs` - Metadata parsing
- `src/parsers/installed/site_packages.rs` - Directory scanning

- Parses METADATA files from `.dist-info` directories (modern format)
- Parses PKG-INFO files from `.egg-info` directories/files (legacy format)
- Extracts package name, version, and `Requires-Dist` dependencies
- Handles complex version specifications and extras
- Links site-packages to parent virtual environments

### 3. Data Models

#### Classification Model

**Module**: `src/models/classification.rs`

- `Classification` enum: Has, Should, Can
- `ClassifiedDependency`: Stores multiple classifications per package
- Version mismatch detection flags
- Application root linking
- Dependency relationship tracking

#### Application Model

**Module**: `src/models/application.rs`

- Represents a project/application root
- Links dependencies to their declaring application
- Tracks manifest file location

#### Dependency Tree Model

**Module**: `src/models/dependency_tree.rs`

- `DependencyNode`: Hierarchical dependency representation
- `DependencyTree`: Complete tree for an application
- Direct vs transitive dependency marking
- Recursive tree traversal and search

#### Installed Package Model

**Module**: `src/models/installed_package.rs`

- `InstalledPackage`: Represents a physically installed package
- `DependencySpec`: Dependency name and version constraint
- Stores installation path and ecosystem

## Use Cases

### Supply Chain Security

Identify which systems have vulnerable packages actually installed:

```bash
# Find all installed packages
scanner --scan-mode installed-only

# Check for specific vulnerable packages
scanner --vuln-list vulnerabilities.txt
```bash

```

**Priority Sorting**:

- HAS (highest priority) - Package is installed, immediate risk
- SHOULD - Package is locked, likely to be installed
- CAN - Package is declared, may or may not be installed

### Dependency Auditing

Understand the complete dependency landscape:

```bash
# Full scan with all three classifications
scanner --scan-mode full

# Generate JSON tree showing dependency relationships
scanner --format json --output dependencies.json
```bash

```

### Version Mismatch Detection

Find discrepancies between installed, locked, and declared versions:

```bash
scanner --scan-mode full
# Check output for version_mismatch and constraint_violation flags
```bash

```

**Mismatch Types**:

- **Version Mismatch**: HAS version differs from SHOULD version
- **Constraint Violation**: SHOULD version doesn't satisfy CAN version range

### Virtual Environment Analysis

Track Python packages across multiple virtual environments:

```bash
scanner --dir /path/to/project
# Automatically detects and scans .venv, venv, etc.
# Links packages to their virtual environment root
```bash

```

## Architecture

### Scan Modes

1. **Full** (default): Scans manifests, lockfiles, and installed packages
2. **Installed-Only**: Only scans installation directories
3. **Declared-Only**: Only scans manifests and lockfiles

### Exclusion Behavior

By default, installation directories are excluded from traversal to avoid redundant scanning:

- `node_modules/` - Detected but not traversed
- `site-packages/`, `dist-packages/` - Detected but not traversed
- `.venv`, `venv`, `env` - Detected but not traversed

Use `--include-install-dirs` to enable deep traversal (for nested installations).

## Implementation Status

### Completed (Tasks 1-4)

✅ **Task 1**: Core data models

- Classification enum and ClassifiedDependency
- Application model for root tracking
- DependencyTree for hierarchical representation
- InstalledPackage model

✅ **Task 2**: Installation directory detection

- node_modules detection
- site-packages/dist-packages detection
- Virtual environment detection
- Scan mode integration

✅ **Task 3**: Node.js installed package parser

- package.json parsing from node_modules
- Scoped package support
- Nested node_modules support
- Comprehensive unit tests

✅ **Task 4**: Python installed package parser

- METADATA file parsing (.dist-info)
- PKG-INFO file parsing (.egg-info)
- Requires-Dist dependency extraction
- Virtual environment path tracking
- Comprehensive unit tests

### Remaining (Tasks 5-14)

⏳ **Task 5**: Analyzer - Classifier

- Group dependencies by name and ecosystem
- Assign HAS/SHOULD/CAN classifications
- Detect version mismatches

⏳ **Task 6**: Analyzer - Application linker

- Link installed packages to application roots
- Find nearest manifest file

⏳ **Task 7**: Analyzer - Dependency tree builder

- Build hierarchical dependency trees
- Handle circular dependencies

⏳ **Task 8**: Analyzer - Vulnerability filter

- Parse vulnerability list files
- Filter and sort by priority

⏳ **Task 9**: Enhanced CSV output

- 15 columns with all classification data
- Version mismatch flags

⏳ **Task 10**: JSON tree output

- Hierarchical format
- Nested dependencies

⏳ **Task 11**: CLI updates

- `--scan-mode` flag
- `--format` flag
- `--vuln-list` flag
- `--output` flag

⏳ **Task 12**: Main integration

- Wire all components together
- Error handling

⏳ **Task 13**: Integration tests

- End-to-end testing
- Test fixtures

⏳ **Task 14**: Documentation

- README updates
- Rustdoc comments

## Testing

All implemented modules include comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run specific module tests
cargo test classification
cargo test install_dirs
cargo test node_modules
cargo test metadata
cargo test site_packages
```bash

```

Current test coverage: 54 passing tests across all modules.

## Performance Considerations

- **Parallel Processing**: Installation directory scanning uses parallel traversal
- **Lazy Evaluation**: Only parses files when needed
- **Smart Exclusions**: Avoids redundant scanning of installation directories
- **Efficient Parsing**: Streaming parsers for large files

## Future Enhancements

- Rust installed package detection (target/ directory analysis)
- Dependency graph visualization
- Security vulnerability database integration
- Incremental scanning for large monorepos
- Database backend for persistent storage
