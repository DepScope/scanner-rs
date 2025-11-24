# Requirements Document

## Introduction

This document specifies requirements for extending the Scanner to detect actually installed packages in filesystem locations (node_modules, site-packages, virtual environments) and establish dependency relationships between applications and their dependencies.
The system will classify dependencies using a three-tier priority system (HAS, SHOULD, CAN) to support supply chain security analysis and vulnerability tracking.

## Glossary

- **Scanner**: The Rust CLI application that analyzes filesystem for package dependencies
- **Installed Package**: A package physically present in a package installation directory (node_modules, site-packages, etc.)
- **Manifest Dependency**: A package declared in a project manifest file with a version range or specification
- **Locked Dependency**: A package version specified in a lock file representing the intended installation
- **HAS Classification**: Indicates a package is physically installed in the filesystem
- **SHOULD Classification**: Indicates a package version is specified in a lock file
- **CAN Classification**: Indicates a package is declared in a manifest with a version range
- **Dependency Tree**: A hierarchical representation showing which packages depend on other packages
- **Virtual Environment**: An isolated Python environment (venv, virtualenv, pyenv, conda)
- **Node Environment**: A Node.js environment manager (nvm, n, volta)
- **Application Root**: The top-level directory containing a project's manifest file
- **Transitive Dependency**: A package required by another dependency, not directly declared

## Requirements

### Requirement 1

**User Story:** As a security analyst investigating a supply chain attack, I want the Scanner to detect all physically installed packages in node_modules directories, so that I can identify which systems have vulnerable packages actually present.

#### Acceptance Criteria

1. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify node_modules directories as Node.js installation locations
2. WHEN the Scanner finds a node_modules directory, THE Scanner SHALL read the package.json file from each subdirectory to extract installed package names and versions
3. WHEN the Scanner extracts an installed package, THE Scanner SHALL record the classification as HAS
4. WHEN the Scanner extracts an installed package, THE Scanner SHALL record the absolute filesystem path to the package directory
5. WHEN the Scanner processes installed packages, THE Scanner SHALL handle nested node_modules directories (for transitive dependencies)

### Requirement 2

**User Story:** As a security analyst investigating a supply chain attack, I want the Scanner to detect all physically installed Python packages in site-packages directories, so that I can identify which systems have vulnerable packages actually present.

#### Acceptance Criteria

1. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify site-packages directories as Python installation locations
2. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify dist-packages directories as Python installation locations
3. WHEN the Scanner finds a site-packages directory, THE Scanner SHALL read package metadata from .dist-info directories to extract installed package names and versions
4. WHEN the Scanner finds a site-packages directory, THE Scanner SHALL read package metadata from .egg-info directories to extract installed package names and versions
5. WHEN the Scanner extracts an installed Python package, THE Scanner SHALL record the classification as HAS
6. WHEN the Scanner extracts an installed Python package, THE Scanner SHALL record the absolute filesystem path to the package directory

### Requirement 3

**User Story:** As a developer managing multiple Python environments, I want the Scanner to detect packages in virtual environments, so that I can audit environment-specific installations.

#### Acceptance Criteria

1. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify directories containing pyvenv.cfg files as Python virtual environments
2. WHEN the Scanner identifies a virtual environment, THE Scanner SHALL scan the environment's site-packages directory for installed packages
3. WHEN the Scanner identifies a virtual environment, THE Scanner SHALL record the virtual environment path in the output
4. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify .venv directories as potential virtual environments
5. WHEN the Scanner traverses a directory tree, THE Scanner SHALL identify venv directories as potential virtual environments

### Requirement 4

**User Story:** As a security analyst, I want the Scanner to link installed packages to their declaring application, so that I can understand which application is responsible for each installation.

#### Acceptance Criteria

1. WHEN the Scanner finds an installed package, THE Scanner SHALL search parent directories for the nearest manifest file (package.json, pyproject.toml, Cargo.toml)
2. WHEN the Scanner identifies a manifest file, THE Scanner SHALL record it as the Application Root for the installed package
3. WHEN the Scanner links an installed package to an application, THE Scanner SHALL include the application name in the output
4. WHEN the Scanner links an installed package to an application, THE Scanner SHALL include the relative path from application root to the installed package

### Requirement 5

**User Story:** As a developer analyzing dependencies, I want the Scanner to classify each package with HAS, SHOULD, or CAN status, so that I can understand the relationship between declared, locked, and installed versions.

#### Acceptance Criteria

1. WHEN a package is found in an installation directory (node_modules, site-packages), THE Scanner SHALL assign the classification HAS
2. WHEN a package version is found in a lock file (package-lock.json, poetry.lock, Cargo.lock), THE Scanner SHALL assign the classification SHOULD
3. WHEN a package is declared in a manifest file (package.json, pyproject.toml, Cargo.toml), THE Scanner SHALL assign the classification CAN
4. WHEN a package has multiple classifications, THE Scanner SHALL record all applicable classifications in the output
5. WHEN outputting classifications, THE Scanner SHALL use the exact strings HAS, SHOULD, and CAN

### Requirement 6

**User Story:** As a security analyst, I want to provide a list of vulnerable packages with versions and receive a prioritized report, so that I can focus on systems where the vulnerable package is actually installed.

#### Acceptance Criteria

1. THE Scanner SHALL accept an input file containing package names and versions to check
2. WHEN the Scanner receives a vulnerability list, THE Scanner SHALL filter output to only include matching packages
3. WHEN the Scanner matches a vulnerable package, THE Scanner SHALL include all three classifications (HAS, SHOULD, CAN) in the output
4. WHEN the Scanner outputs vulnerability matches, THE Scanner SHALL sort results with HAS classification first
5. WHEN the Scanner outputs vulnerability matches, THE Scanner SHALL include the application root for each match

### Requirement 7

**User Story:** As a developer analyzing dependencies, I want the Scanner to build a dependency tree showing which packages depend on other packages, so that I can understand transitive dependency relationships.

#### Acceptance Criteria

1. WHEN the Scanner parses a package.json file in node_modules, THE Scanner SHALL extract the dependencies field to identify direct dependencies
2. WHEN the Scanner parses a METADATA file in site-packages, THE Scanner SHALL extract the Requires-Dist field to identify direct dependencies
3. WHEN the Scanner identifies a dependency relationship, THE Scanner SHALL record the parent package and child package names
4. WHEN the Scanner identifies a dependency relationship, THE Scanner SHALL record the version constraint for the dependency
5. WHEN outputting dependency relationships, THE Scanner SHALL indicate whether the dependency is direct or transitive

### Requirement 8

**User Story:** As a security analyst, I want the Scanner to detect version mismatches between installed, locked, and declared versions, so that I can identify configuration drift or installation issues.

#### Acceptance Criteria

1. WHEN a package has both HAS and SHOULD classifications, THE Scanner SHALL compare the installed version with the locked version
2. WHEN installed and locked versions differ, THE Scanner SHALL record a version mismatch flag in the output
3. WHEN a package has both SHOULD and CAN classifications, THE Scanner SHALL verify the locked version satisfies the manifest version range
4. WHEN a locked version does not satisfy the manifest range, THE Scanner SHALL record a constraint violation flag in the output
5. WHEN outputting mismatches, THE Scanner SHALL include all three versions (installed, locked, declared) for comparison

### Requirement 9

**User Story:** As a developer using the Scanner, I want enhanced CSV output that includes installation status and dependency relationships, so that I can analyze the data in spreadsheet tools.

#### Acceptance Criteria

1. WHEN generating CSV output, THE Scanner SHALL include a column for HAS classification (boolean or version)
2. WHEN generating CSV output, THE Scanner SHALL include a column for SHOULD classification (boolean or version)
3. WHEN generating CSV output, THE Scanner SHALL include a column for CAN classification (boolean or version range)
4. WHEN generating CSV output, THE Scanner SHALL include a column for application root path
5. WHEN generating CSV output, THE Scanner SHALL include a column for installed package path
6. WHEN generating CSV output, THE Scanner SHALL include a column for parent package (for dependency tree)
7. WHEN generating CSV output, THE Scanner SHALL include a column for version mismatch flag

### Requirement 10

**User Story:** As a developer using the Scanner, I want an alternative JSON output format for dependency trees, so that I can programmatically analyze hierarchical relationships.

#### Acceptance Criteria

1. THE Scanner SHALL support a --format flag accepting values csv or json
2. WHEN the format is json, THE Scanner SHALL output a JSON array of application objects
3. WHEN outputting JSON, THE Scanner SHALL nest installed packages under their application root
4. WHEN outputting JSON, THE Scanner SHALL represent dependency trees as nested objects
5. WHEN outputting JSON, THE Scanner SHALL include all classification data (HAS, SHOULD, CAN) for each package

### Requirement 11

**User Story:** As a developer running the Scanner, I want to exclude virtual environments and node_modules from directory traversal by default, so that scans complete quickly without redundant data.

#### Acceptance Criteria

1. THE Scanner SHALL exclude .venv directories from directory traversal by default
2. THE Scanner SHALL exclude venv directories from directory traversal by default
3. THE Scanner SHALL exclude env directories from directory traversal by default
4. THE Scanner SHALL exclude node_modules directories from directory traversal by default
5. THE Scanner SHALL provide a --include-install-dirs flag to scan inside installation directories
6. WHEN --include-install-dirs is enabled, THE Scanner SHALL traverse node_modules and virtual environment directories

### Requirement 12

**User Story:** As a security analyst, I want to scan only for installed packages without parsing manifests or lock files, so that I can quickly inventory what is actually present on a system.

#### Acceptance Criteria

1. THE Scanner SHALL support a --scan-mode flag accepting values full, installed-only, or declared-only
2. WHEN scan-mode is installed-only, THE Scanner SHALL only detect HAS classifications
3. WHEN scan-mode is declared-only, THE Scanner SHALL only detect SHOULD and CAN classifications
4. WHEN scan-mode is full, THE Scanner SHALL detect all three classifications
5. WHEN scan-mode is installed-only, THE Scanner SHALL skip parsing manifest and lock files
