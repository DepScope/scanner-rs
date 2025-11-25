# Scanner

A high-performance multi-language dependency scanner for Python, Node.js, and Rust ecosystems. Scanner recursively traverses directories to identify package management files, parse dependencies, and analyze installed packages for supply chain security.

## Features

- **Supply Chain Security**: Detect infected packages from attacks like Shai Hulud (400+ compromised npm packages)
- **Multi-Ecosystem Support**: Scans Python, Node.js/TypeScript, and Rust projects
- **Comprehensive File Format Coverage**:
  - **Node.js**: package.json, yarn.lock, package-lock.json, pnpm-lock.yaml, node_modules
  - **Python**: pyproject.toml, requirements.txt, poetry.lock, uv.lock, site-packages
  - **Rust**: Cargo.toml, Cargo.lock
- **Installed Package Detection**: Scans node_modules and site-packages to find actually installed packages
- **Virtual Environment Support**: Detects and scans Python virtual environments (venv, .venv, pyenv)
- **HAS/SHOULD/CAN Classification**: Three-tier system for dependency analysis:
  - **HAS**: Package is physically installed (found in node_modules, site-packages)
  - **SHOULD**: Package version is in a lock file (intended installation)
  - **CAN**: Package is declared in a manifest (allowed versions)
- **Dependency Tree Building**: Constructs hierarchical dependency relationships
- **Application Linking**: Links installed packages to their declaring applications
- **Supply Chain Security**: Identify which systems have vulnerable packages actually installed
- **Parallel Processing**: Fast scanning with configurable thread pools
- **Detailed Reporting**: CSV output with dependency type, ecosystem, and source file information
- **Flexible Filtering**: Filter results by ecosystem

## Installation

### From Release (Recommended)

Download the latest binary for your platform from the [releases page](https://github.com/DepScope/scanner-rs/releases):

```bash
# macOS Apple Silicon (M1/M2)
curl -L https://github.com/DepScope/scanner-rs/releases/latest/download/scanner-darwin-aarch64.tar.gz | tar xz

# macOS Intel
curl -L https://github.com/DepScope/scanner-rs/releases/latest/download/scanner-darwin-x86_64.tar.gz | tar xz

# Linux x86_64
curl -L https://github.com/DepScope/scanner-rs/releases/latest/download/scanner-linux-x86_64.tar.gz | tar xz

# Make executable and move to PATH
chmod +x scanner
sudo mv scanner /usr/local/bin/
```bash

```

### From Source

```bash
cargo build --release
```bash

```

The binary will be available at `target/release/scanner`.

## Usage

### Basic Scan

Scan the current directory for all dependencies (declared and installed):

```bash
scanner
```

### Scan Specific Directory

```bash
scanner --dir /path/to/project
```

### Scan Modes

**Full Scan** (default): Scans both declared dependencies and installed packages

```bash
scanner --scan-mode full
```

**Installed Only**: Only scan for physically installed packages (HAS classification)

```bash
scanner --scan-mode installed-only
```

**Declared Only**: Only scan manifest and lock files (SHOULD and CAN classifications)

```bash
scanner --scan-mode declared-only
```

### Output Formats

**CSV Output** (default): Flat table with all classification data

```bash
scanner --format csv --output results.csv
```

**JSON Output**: Hierarchical dependency trees

```bash
scanner --format json --output results.json
```

### Filter by Ecosystem

Scan only Node.js dependencies:

```bash
scanner --ecosystem node
```

Scan only Python dependencies:

```bash
scanner --ecosystem python
```

Scan only Rust dependencies:

```bash
scanner --ecosystem rust
```

### Supply Chain Security: Shai Hulud Detection

Scan your entire system for infected packages from the Shai Hulud supply chain attack:

```bash
# Scan your home directory for infected packages
scanner --infected-list shai-hulud2.csv --dir ~ --output shai-hulud-scan.csv

# Scan with verbose output to see progress
scanner --infected-list shai-hulud2.csv --dir ~ --verbose --output shai-hulud-scan.csv
```

For detailed analysis and remediation guidance, see [SCAN_SUPPLYCHAIN.md](docs/SCAN_SUPPLYCHAIN.md).

### Infected Package Detection

Scan for infected packages (ransomware/worm) using a CSV list with multiple versions per package:

```bash
# Create infected package list (CSV format: package,version1 | version2 | version3)
cat > infected.csv << EOF
webpack-loader-httpfile,0.2.1
wellness-expert-ng-gallery,5.1.1
wenk,1.0.9 | 1.0.10
zapier-async-storage,1.0.3 | 1.0.2 | 1.0.1
zapier-platform-cli,18.0.4 | 18.0.3 | 18.0.2
lodash,4.17.21
react,18.2.0
EOF

# Run scan with infected package detection
scanner --infected-list infected.csv --output results.csv
```

The scanner will add a `security` column to the CSV output with three possible values:

- **NONE**: Package is not in the infected list
- **MATCH_PACKAGE**: Package name matches but version is different (not infected)
- **INFECTED**: Package name and version match the infected list

Results are automatically sorted by priority:

1. **HAS** (highest priority - actually installed)
2. **SHOULD** (in lock file)
3. **CAN** (declared in manifest)

### Configure Thread Count

```bash
scanner --jobs 8
```

### Verbose Output

```bash
scanner --verbose
```

## Classification System (HAS/SHOULD/CAN)

Scanner provides a three-tier classification system for comprehensive dependency analysis:

### Classifications

- **HAS**: Package is physically installed in the filesystem
  - Found in: `node_modules/`, `site-packages/`, `.venv/`
  - Indicates: What's actually present on the system
  - Priority: Highest (most critical for security)

- **SHOULD**: Package version is specified in a lock file
  - Found in: `package-lock.json`, `yarn.lock`, `poetry.lock`, `Cargo.lock`
  - Indicates: What should be installed (intended state)
  - Priority: Medium

- **CAN**: Package is declared in a manifest with a version range
  - Found in: `package.json`, `pyproject.toml`, `Cargo.toml`
  - Indicates: What versions are allowed
  - Priority: Lowest

### Why This Matters

A package can have multiple classifications simultaneously:

```text
react:
  HAS: 18.2.0 (installed in node_modules)
  SHOULD: 18.2.0 (locked in package-lock.json)
  CAN: ^18.0.0 (declared in package.json)
```

This helps identify:

- **Version mismatches**: HAS ≠ SHOULD (drift from lock file)
- **Constraint violations**: SHOULD doesn't satisfy CAN range
- **Security exposure**: Vulnerable packages that are actually installed (HAS) vs just declared

## Output Formats

### CSV Output (Enhanced)

The enhanced CSV format includes all classification data:

```csv
package_name,ecosystem,application_name,application_root,has_version,has_path,should_version,should_path,can_version,can_path,version_mismatch,constraint_violation,parent_package,is_direct,dependency_count
react,node,myapp,/app,18.2.0,/app/node_modules/react,18.2.0,/app/package-lock.json,^18.0.0,/app/package.json,false,false,,true,2
```

Columns:

- `package_name`: Package name
- `ecosystem`: node, python, or rust
- `application_name`: Name of the declaring application
- `application_root`: Path to application root directory
- `has_version`: Installed version (if present)
- `has_path`: Path to installed package
- `should_version`: Locked version (if present)
- `should_path`: Path to lock file
- `can_version`: Version range from manifest (if present)
- `can_path`: Path to manifest file
- `version_mismatch`: true if HAS ≠ SHOULD
- `constraint_violation`: true if SHOULD doesn't satisfy CAN
- `parent_package`: Parent dependency (for tree structure)
- `is_direct`: true if direct dependency
- `dependency_count`: Number of dependencies this package has
- `security`: NONE, MATCH_PACKAGE, or INFECTED (when using --infected-list)

### JSON Output

Hierarchical dependency trees with full classification data:

```json
{
  "applications": [
    {
      "name": "myapp",
      "root_path": "/app",
      "manifest_path": "/app/package.json",
      "ecosystem": "node",
      "dependencies": [
        {
          "name": "react",
          "classifications": {
            "has": {
              "version": "18.2.0",
              "path": "/app/node_modules/react"
            },
            "should": {
              "version": "18.2.0",
              "path": "/app/package-lock.json"
            },
            "can": {
              "version": "^18.0.0",
              "path": "/app/package.json"
            }
          },
          "version_mismatch": false,
          "is_direct": true,
          "security": "INFECTED",
          "dependencies": [...]
        }
      ]
    }
  ]
}
```

**Note**: The `security` field is only included when using `--infected-list` flag.

### Use Cases

**Supply Chain Security**: Identify which systems have vulnerable packages actually installed vs merely declared:

```bash
# Find all installed packages
scanner --scan-mode installed-only

# Check for specific vulnerable packages
scanner --vuln-list vulnerabilities.txt
```bash

```

**Dependency Auditing**: Understand the complete dependency landscape:

```bash
# Full scan with all three classifications
scanner --scan-mode full

# Generate JSON tree showing dependency relationships
scanner --format json --output dependencies.json
```bash

```

**Version Mismatch Detection**: Find discrepancies between installed, locked, and declared versions:

```bash
scanner --scan-mode full
# Check output for version_mismatch and constraint_violation flags
```bash

```

## Supported File Formats

### Node.js Ecosystem

**Manifest Files** (CAN - declared dependencies):

- `package.json` - npm/yarn/pnpm package manifest

**Lockfiles** (SHOULD - resolved versions):

- `yarn.lock` - Yarn v1/v2 lockfile
- `package-lock.json` - npm lockfile (v1/v2/v3)
- `pnpm-lock.yaml` - pnpm lockfile

**Installed Packages** (HAS - actually installed):

- `node_modules/` - Installed Node.js packages
  - Reads package.json from each subdirectory
  - Handles scoped packages (@org/package)
  - Supports nested node_modules (transitive dependencies)

### Python Ecosystem

**Manifest Files** (CAN):

- `pyproject.toml` - PEP 621 and Poetry project files
- `requirements.txt` - pip requirements

**Lockfiles** (SHOULD):

- `poetry.lock` - Poetry lockfile
- `uv.lock` - uv lockfile

**Installed Packages** (HAS):

- `site-packages/` - Installed Python packages
  - Parses METADATA from .dist-info directories
  - Parses PKG-INFO from .egg-info directories
  - Detects virtual environments (venv, .venv, pyenv)
  - Tracks virtual environment paths

### Rust Ecosystem

**Manifest Files** (CAN):

- `Cargo.toml` - Cargo package manifest

**Lockfiles** (SHOULD):

- `Cargo.lock` - Cargo lockfile

## Excluded Directories

By default, Scanner excludes installation directories from traversal to avoid redundant scanning:

- `node_modules` - Detected but not traversed (unless --include-install-dirs)
- `site-packages` - Detected but not traversed
- `dist-packages` - Detected but not traversed
- `.venv`, `venv`, `env` - Virtual environments (detected but not traversed)
- `.nx` - Nx workspace cache
- `target` - Rust build directory
- `.git` - Git repository data

To include installation directories in traversal (for deep nested scanning):

```bash
scanner --include-install-dirs
```bash

```

## Architecture

Scanner uses a modular parser-based architecture:

- **Indexer**: Fast parallel filesystem traversal
- **Parser Registry**: Extensible parser system for different file formats
- **Parsers**: Dedicated parsers for each file format
- **Models**: Structured data representation
- **Output**: CSV generation

## Development

### Setup Pre-commit Hooks

Install pre-commit hooks for automatic code quality checks:

```bash
./setup-hooks.sh
```bash

```

This sets up automatic checks for:

- Code formatting (rustfmt)
- Linting (clippy)
- Compilation
- Tests
- Code analytics

See [PRE_COMMIT_HOOKS.md](PRE_COMMIT_HOOKS.md) for details.

### Run Tests

```bash
cargo test
# Or use make
make test
```bash

```

### Code Quality Checks

```bash
# Run all pre-commit checks
make pre-commit

# Individual checks
make fmt      # Format code
make clippy   # Lint code
make check    # Check compilation
```bash

```

### Build Documentation

```bash
cargo doc --open
```bash

```

### Run with Debug Output

```bash
cargo run -- --verbose
```bash

```

## Performance

Scanner uses Rayon for parallel processing, automatically detecting the optimal number of threads based on your CPU. For large monorepos, you can adjust the thread count:

```bash
scanner --jobs 16
```bash

```

## License

MIT OR Apache-2.0

## Releasing

### Quick Release

```bash
# Create a release (automatically builds all possible binaries)
./release-all.sh --patch   # 0.1.0 → 0.1.1
./release-all.sh --minor   # 0.1.0 → 0.2.0
./release-all.sh --major   # 0.1.0 → 1.0.0

# Or use make
make release-patch
make release-minor
make release-major
```bash

```

The unified release script (`release-all.sh`) will:

1. Run pre-flight checks (git, GitHub CLI, uncommitted changes)
2. Bump version in `Cargo.toml`
3. Create and push a git tag
4. Build binaries for all available architectures
5. Create a GitHub release with all binaries

**Binaries created**:

- Always: Native binary for your current architecture
- With rustup: Additional binaries for other architectures (x86_64 + aarch64)

See [RELEASING.md](RELEASING.md) for complete documentation.

## Contributing

Contributions are welcome! Please follow these steps:

1. **Setup pre-commit hooks**:

   ```bash
   ./setup-hooks.sh
   ```

2. **Make your changes** and ensure all checks pass:

   ```bash
   make pre-commit
   ```

3. **Commit with conventional format**:

   ```bash
   git commit -m "feat(module): description"
   ```

4. **Submit a pull request**

Pre-commit hooks will automatically check:

- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Compilation (`cargo check`)
- Tests (`cargo test`)

See [PRE_COMMIT_HOOKS.md](PRE_COMMIT_HOOKS.md) for details.

## Quick Reference

### Supply Chain Security Scanning

```bash
# Scan for Shai Hulud infected packages
scanner --infected-list shai-hulud2.csv --dir ~ --output scan.csv

# View infected packages
grep "INFECTED" scan.csv

# Visualize results in dashboard
cd dashboard && streamlit run app.py

# Detailed analysis guide
See docs/SCAN_SUPPLYCHAIN.md
```

### Common Commands

```bash
# Full scan with all features
scanner --scan-mode full --format csv --output results.csv

# Scan specific ecosystem
scanner --ecosystem node --dir /path/to/project

# JSON output for automation
scanner --format json --output results.json

# Verbose output for debugging
scanner --verbose --dir /path/to/project

# Visualize results
cd dashboard && streamlit run app.py
```

## Dashboard

Scanner includes an interactive Streamlit dashboard for visualizing scan results. The dashboard provides:

- **Multi-page Analysis**: Separate views for Node.js, Python, and Rust packages
- **Interactive Visualizations**: Charts and graphs for package distribution and version analysis
- **Infected Package Detection**: Visual identification of compromised packages
- **Version Consistency Analysis**: Identify packages with multiple versions across your codebase
- **CSV File Management**: Upload, browse, or manually specify CSV files to analyze

### Installation

```bash
cd dashboard
pip install -r requirements.txt
```

### Running the Dashboard

```bash
# From the dashboard directory
streamlit run app.py

# Or specify a custom CSV file
streamlit run app.py -- --csv-path /path/to/results.csv
```

The dashboard will open in your browser at `http://localhost:8501`.

### Features

**Main Overview Page**:

- Total package counts and unique package metrics
- Most frequently used packages across all ecosystems
- Infected package analysis with distribution charts
- Package version distribution for selected packages
- Raw data viewer with CSV export

**Ecosystem-Specific Pages**:

- **Node Packages** (📦): Node.js/npm package analysis
- **Python Packages** (🐍): Python package analysis
- **Rust Packages** (🦀): Rust crate analysis

Each page includes:

- Top 20 most used packages
- Version consistency analysis
- Framework/library detection
- Detailed package information with expandable views

**CSV File Loading Options**:

1. **Upload**: Drag and drop or browse for a CSV file
2. **Browse**: Select from available CSV files in the workspace
3. **Manual Path**: Enter a file path directly

### Dashboard Requirements

- Python 3.8+
- Streamlit 1.40+
- Pandas
- Plotly

See `dashboard/requirements.txt` for complete dependencies.

## Documentation

### User Guides

- [Supply Chain Security Guide](docs/SCAN_SUPPLYCHAIN.md) - Detecting and remediating infected packages
- [Infected Package Detection](docs/INFECTED_PACKAGE_DETECTION.md) - Technical details on detection system
- [Implementation Summary](docs/IMPLEMENTATION_SUMMARY.md) - HAS/SHOULD/CAN classification system

### Developer Guides

- [Build and Release Guide](docs/BUILD_AND_RELEASE.md) - Building for all platforms and creating releases
- [Cross-Compilation](docs/CROSS_COMPILE.md) - Detailed cross-compilation guide
- [Docker Builds](docs/DOCKER_BUILDS.md) - Docker-based Linux builds
- [Pre-commit Hooks](docs/PRE_COMMIT_HOOKS.md) - Development workflow and git hooks
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions

See [docs/README.md](docs/README.md) for complete documentation index.
