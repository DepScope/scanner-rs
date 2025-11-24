# Requirements Document

## Introduction

This document specifies requirements for transforming the existing npm-focused scanner into a comprehensive multi-language dependency scanner.
The system will identify, parse, and analyze package dependencies across Python, Node.js/TypeScript, Rust, and future ecosystems (Java).
It will distinguish between declared dependencies (in project manifests) and resolved/installed versions (in lock files), providing a complete view of dependency usage across a filesystem.

## Glossary

- **Scanner**: The Rust CLI application that analyzes filesystem for package dependencies
- **Manifest File**: A project configuration file declaring possible dependencies (e.g., package.json, Cargo.toml, pyproject.toml)
- **Lock File**: A file recording exact resolved/installed dependency versions (e.g., yarn.lock, Cargo.lock, poetry.lock)
- **Parser Module**: A dedicated Rust module responsible for parsing a specific file format
- **Dependency Record**: A structured data representation containing package name, version, source file, and dependency type
- **Filesystem Indexer**: The component that discovers relevant package system files during traversal
- **SemVer**: Semantic versioning specification for version comparison and range resolution

## Requirements

### Requirement 1

**User Story:** As a developer auditing a monorepo, I want the Scanner to identify all package system files across Python, Node.js, and Rust ecosystems, so that I can understand the complete dependency landscape.

#### Acceptance Criteria

1. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify package.json files as Node.js manifest files
2. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify pyproject.toml files as Python manifest files
3. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify requirements.txt files as Python manifest files
4. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify Cargo.toml files as Rust manifest files
5. WHEN the Scanner traverses a directory tree, THE Scanner SHALL exclude node_modules directories from traversal
6. WHEN the Scanner traverses a directory tree, THE Scanner SHALL exclude .nx directories from traversal
7. WHEN the Scanner traverses a directory tree, THE Scanner SHALL exclude target directories from traversal

### Requirement 2

**User Story:** As a developer analyzing dependencies, I want the Scanner to identify all lock files for each package manager, so that I can see exactly which versions are installed.

#### Acceptance Criteria

1. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify yarn.lock files as Yarn lock files
2. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify package-lock.json files as npm lock files
3. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify pnpm-lock.yaml files as pnpm lock files
4. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify poetry.lock files as Poetry lock files
5. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify uv.lock files as uv lock files
6. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify Cargo.lock files as Rust lock files
7. WHEN the Scanner discovers a directory with package files, THE Scanner SHALL identify bun.lock files as Bun lock files

### Requirement 3

**User Story:** As a developer maintaining the Scanner codebase, I want each file format to have a dedicated parser module, so that the code is maintainable and extensible.

#### Acceptance Criteria

1. THE Scanner SHALL implement a separate Rust module for parsing package.json files
2. THE Scanner SHALL implement a separate Rust module for parsing pyproject.toml files
3. THE Scanner SHALL implement a separate Rust module for parsing requirements.txt files
4. THE Scanner SHALL implement a separate Rust module for parsing Cargo.toml files
5. THE Scanner SHALL implement a separate Rust module for parsing yarn.lock files
6. THE Scanner SHALL implement a separate Rust module for parsing package-lock.json files
7. THE Scanner SHALL implement a separate Rust module for parsing pnpm-lock.yaml files
8. THE Scanner SHALL implement a separate Rust module for parsing poetry.lock files
9. THE Scanner SHALL implement a separate Rust module for parsing uv.lock files
10. THE Scanner SHALL implement a separate Rust module for parsing Cargo.lock files

### Requirement 4

**User Story:** As a developer using the Scanner, I want manifest parsers to extract declared dependencies with their version specifications, so that I can see what versions are allowed by the project configuration.

#### Acceptance Criteria

1. WHEN parsing a package.json file, THE Scanner SHALL extract all entries from the dependencies object with their version ranges
2. WHEN parsing a package.json file, THE Scanner SHALL extract all entries from the devDependencies object with their version ranges
3. WHEN parsing a pyproject.toml file, THE Scanner SHALL extract all entries from the dependencies array with their version specifications
4. WHEN parsing a pyproject.toml file, THE Scanner SHALL extract all entries from the tool.poetry.dependencies section with their version specifications
5. WHEN parsing a requirements.txt file, THE Scanner SHALL extract package names and version specifications from each line
6. WHEN parsing a Cargo.toml file, THE Scanner SHALL extract all entries from the dependencies section with their version requirements

### Requirement 5

**User Story:** As a developer using the Scanner, I want lock file parsers to extract exact resolved versions, so that I can see precisely which versions are installed.

#### Acceptance Criteria

1. WHEN parsing a yarn.lock file, THE Scanner SHALL extract exact resolved versions for each package entry
2. WHEN parsing a package-lock.json file, THE Scanner SHALL extract exact resolved versions for each package entry
3. WHEN parsing a pnpm-lock.yaml file, THE Scanner SHALL extract exact resolved versions for each package entry
4. WHEN parsing a poetry.lock file, THE Scanner SHALL extract exact resolved versions for each package entry
5. WHEN parsing a uv.lock file, THE Scanner SHALL extract exact resolved versions for each package entry
6. WHEN parsing a Cargo.lock file, THE Scanner SHALL extract exact resolved versions for each package entry

### Requirement 6

**User Story:** As a developer maintaining the Scanner, I want each parser module to have comprehensive unit tests, so that I can verify correctness and prevent regressions.

#### Acceptance Criteria

1. WHEN a parser module is implemented, THE Scanner SHALL include unit tests that verify parsing of valid input files
2. WHEN a parser module is implemented, THE Scanner SHALL include unit tests that verify handling of malformed input files
3. WHEN a parser module is implemented, THE Scanner SHALL include unit tests that verify extraction of dependency names
4. WHEN a parser module is implemented, THE Scanner SHALL include unit tests that verify extraction of version specifications
5. WHEN a parser module is implemented, THE Scanner SHALL include test fixture files representing real-world examples

### Requirement 7

**User Story:** As a developer analyzing dependencies, I want the Scanner to use appropriate SemVer libraries for each ecosystem, so that version comparisons are accurate.

#### Acceptance Criteria

1. WHEN comparing Node.js package versions, THE Scanner SHALL use the node-semver crate for version parsing and comparison
2. WHEN comparing Rust package versions, THE Scanner SHALL use the semver crate for version parsing and comparison
3. WHEN comparing Python package versions, THE Scanner SHALL use PEP 440 compliant version parsing

### Requirement 8

**User Story:** As a developer using the Scanner, I want output that distinguishes between declared and resolved dependencies, so that I can identify version mismatches.

#### Acceptance Criteria

1. WHEN generating output, THE Scanner SHALL include a field indicating whether a dependency is declared in a manifest file
2. WHEN generating output, THE Scanner SHALL include a field indicating whether a dependency is resolved in a lock file
3. WHEN generating output, THE Scanner SHALL include the source file path for each dependency record
4. WHEN generating output, THE Scanner SHALL include the dependency type (dependencies vs devDependencies vs build dependencies)

### Requirement 9

**User Story:** As a developer running the Scanner, I want the filesystem indexing to be fast and parallelized, so that I can scan large codebases efficiently.

#### Acceptance Criteria

1. THE Scanner SHALL use parallel processing for file discovery operations
2. THE Scanner SHALL use parallel processing for file parsing operations
3. WHEN the Scanner processes files in parallel, THE Scanner SHALL use a configurable thread pool size
4. WHEN the Scanner processes files in parallel, THE Scanner SHALL collect results in a thread-safe manner

### Requirement 10

**User Story:** As a developer extending the Scanner, I want a clear module structure that separates concerns, so that I can easily add support for new package managers.

#### Acceptance Criteria

1. THE Scanner SHALL organize parser modules under a dedicated parsers directory
2. THE Scanner SHALL define a common trait or interface for all parser modules
3. THE Scanner SHALL implement a registry or factory pattern for selecting appropriate parsers
4. THE Scanner SHALL separate filesystem indexing logic from parsing logic
5. THE Scanner SHALL separate output generation logic from parsing logic
