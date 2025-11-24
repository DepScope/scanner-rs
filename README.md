# Scanner

A high-performance CLI tool for analyzing npm package compatibility across JavaScript/TypeScript projects.

## Description

Scanner recursively scans directories to identify npm packages and verify their presence across different package manager lock files (yarn.lock, package-lock.json, pnpm-lock.yaml) and dependency manifests. Perfect for auditing monorepos, ensuring version consistency, and generating compatibility reports.

## Features

- **Multi-package manager support**: yarn, npm, pnpm
- **Parallel processing**: Configurable thread pool for fast scanning
- **Smart exclusions**: Automatically skips node_modules and .nx directories
- **Flexible scanning**: Scan from any directory with custom start paths
- **CSV export**: Detailed reports for analysis and auditing
- **Real paths**: Outputs absolute paths for unambiguous results

## Installation

### From Source

```bash
git clone <repository-url>
cd scanner
cargo build --release
```

The binary will be available at `target/release/scanner`.

### From GitHub Releases

Download pre-built binaries for your platform from the [Releases](../../releases) page:
- Linux x86_64
- Linux ARM64
- macOS x86_64
- macOS ARM64 (Apple Silicon)

## Usage

### Quick Start

1. Create a `packages.txt` file with packages to check (format: `name@version`):

```
lodash@4.17.21
react@18.2.0
express@4.18.2
```

2. Run the scanner:

```bash
scanner
```

### Command Line Options

```
-d, --dir <DIR>        Directory to start scanning from [default: .]
    --root-only        Only check the root directory
    --list-dirs        Only list directories to be checked
-j, --jobs <JOBS>      Number of worker threads [default: CPU count]
    --no-npm           Skip calling npm (faster, recommended)
-v, --verbose          Enable verbose logging
-h, --help             Print help
-V, --version          Print version
```

### Examples

Scan current directory with verbose output:
```bash
scanner --verbose --no-npm
```

Scan a specific directory with 8 threads:
```bash
scanner --dir /path/to/project --jobs 8
```

List directories that would be scanned:
```bash
scanner --list-dirs
```

Scan only the root directory:
```bash
scanner --root-only --no-npm
```

## Output

### Console Output
Displays found packages with matching versions:
```
/path/to/project:lodash@4.17.21
/path/to/project/packages/app:react@18.2.0
```

### CSV Report
Generated as `output.csv` with columns:
- `package`: Package name
- `version`: Version being checked
- `location`: Absolute path to directory
- `match_package`: Whether package was found (true/false)
- `match_version`: Whether version matches (true/false)

## How It Works

1. **Discovery**: Recursively finds directories containing package.json, lock files, or DEPENDENCIES.json
2. **Preloading**: Reads all lock files once before parallel processing
3. **Parallel Scanning**: Uses rayon to scan packages across directories concurrently
4. **Version Extraction**: Parses lock files using format-specific strategies:
   - yarn.lock: Regex-based parsing
   - package-lock.json: JSON parsing
   - pnpm-lock.yaml: Regex-based parsing
   - DEPENDENCIES.json: JSON parsing
   - npm ls: Shell command (optional)
5. **Reporting**: Generates CSV with match results

## Requirements

- Rust 2021 edition or later
- No runtime dependencies (statically linked binary)

## License

MIT OR Apache-2.0
