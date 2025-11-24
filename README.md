# Scanner

A high-performance multi-language dependency scanner for Python, Node.js, and Rust ecosystems. Scanner recursively traverses directories to identify package management files, parse dependencies, and generate comprehensive reports.

## Features

- **Multi-Ecosystem Support**: Scans Python, Node.js/TypeScript, and Rust projects
- **Comprehensive File Format Coverage**:
  - **Node.js**: package.json, yarn.lock, package-lock.json, pnpm-lock.yaml
  - **Python**: pyproject.toml, requirements.txt, poetry.lock, uv.lock
  - **Rust**: Cargo.toml, Cargo.lock
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
```

### From Source

```bash
cargo build --release
```

The binary will be available at `target/release/scanner`.

## Usage

### Basic Scan

Scan the current directory for all dependencies:

```bash
scanner
```

### Scan Specific Directory

```bash
scanner --dir /path/to/project
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

### Configure Thread Count

```bash
scanner --jobs 8
```

### Verbose Output

```bash
scanner --verbose
```

## Output

Scanner generates an `output.csv` file with the following columns:

- **package**: Package name
- **version**: Version specification (range for manifests, exact for lockfiles)
- **source_file**: Path to the file where the dependency was found
- **dep_type**: Dependency type (runtime, development, peer, optional, build)
- **ecosystem**: Package ecosystem (node, python, rust)
- **file_type**: Whether from manifest or lockfile

### Example Output

```csv
package,version,source_file,dep_type,ecosystem,file_type
react,^18.2.0,/project/package.json,runtime,node,manifest
react,18.2.0,/project/yarn.lock,runtime,node,lockfile
django,^4.2.0,/project/pyproject.toml,runtime,python,manifest
django,4.2.3,/project/poetry.lock,runtime,python,lockfile
serde,1.0,/project/Cargo.toml,runtime,rust,manifest
serde,1.0.188,/project/Cargo.lock,runtime,rust,lockfile
```

## Supported File Formats

### Node.js Ecosystem

**Manifest Files** (declared dependencies):
- `package.json` - npm/yarn/pnpm package manifest

**Lockfiles** (resolved versions):
- `yarn.lock` - Yarn v1/v2 lockfile
- `package-lock.json` - npm lockfile (v1/v2/v3)
- `pnpm-lock.yaml` - pnpm lockfile

### Python Ecosystem

**Manifest Files**:
- `pyproject.toml` - PEP 621 and Poetry project files
- `requirements.txt` - pip requirements

**Lockfiles**:
- `poetry.lock` - Poetry lockfile
- `uv.lock` - uv lockfile

### Rust Ecosystem

**Manifest Files**:
- `Cargo.toml` - Cargo package manifest

**Lockfiles**:
- `Cargo.lock` - Cargo lockfile

## Excluded Directories

Scanner automatically excludes the following directories from traversal:
- `node_modules`
- `.nx`
- `target`
- `.git`
- `.venv`
- `venv`
- `__pycache__`

## Architecture

Scanner uses a modular parser-based architecture:

- **Indexer**: Fast parallel filesystem traversal
- **Parser Registry**: Extensible parser system for different file formats
- **Parsers**: Dedicated parsers for each file format
- **Models**: Structured data representation
- **Output**: CSV generation

## Development

### Run Tests

```bash
cargo test
```

### Build Documentation

```bash
cargo doc --open
```

### Run with Debug Output

```bash
cargo run -- --verbose
```

## Performance

Scanner uses Rayon for parallel processing, automatically detecting the optimal number of threads based on your CPU. For large monorepos, you can adjust the thread count:

```bash
scanner --jobs 16
```

## License

MIT OR Apache-2.0

## Releasing

### Quick Release

```bash
# Check if ready to release
./check-release.sh

# Create a release (native architecture only)
./release.sh --patch   # 0.1.0 → 0.1.1
./release.sh --minor   # 0.1.0 → 0.2.0
./release.sh --major   # 0.1.0 → 1.0.0

# Or use make
make release-patch
make release-minor
make release-major
```

The release script will:
1. Bump version in `Cargo.toml`
2. Create and push a git tag
3. Build binaries for your architecture
4. Cross-compile for other architectures (if rustup is installed)
5. Create a GitHub release with all binaries

**Note**: For cross-compilation (building for multiple architectures), you need rustup. See [CROSS_COMPILE_SETUP.md](CROSS_COMPILE_SETUP.md) for setup instructions.

See [RELEASE.md](RELEASE.md) for detailed documentation.

## Contributing

Contributions are welcome! Please ensure all tests pass before submitting a pull request:

```bash
cargo test
cargo fmt
cargo clippy
```
