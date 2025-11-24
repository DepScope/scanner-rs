---
inclusion: always
---

# Project Structure

## Directory Layout

```
.
├── src/
│   └── main.rs          # Single-file application with all logic
├── Cargo.toml           # Package manifest and dependencies
├── Cargo.lock           # Locked dependency versions
├── packages.txt         # Input file: packages to scan (name@version format)
├── output.csv           # Generated output: scan results
└── README.md            # Documentation
```

## Architecture

This is a single-file CLI application (`src/main.rs`) organized into logical sections:

1. **CLI Definition**: `Args` struct with clap derive macros
2. **Data Structures**: `Preload` struct for caching lock file contents
3. **Directory Discovery**: `find_dirs()` - locates directories with package files
4. **Version Parsing**: `parse_version()` - extracts semantic versions
5. **Lock File Parsers**: Separate functions per format:
   - `get_yarn_versions()` - regex-based yarn.lock parsing
   - `get_package_lock_versions()` - JSON-based package-lock.json parsing
   - `get_pnpm_versions()` - regex-based pnpm-lock.yaml parsing
   - `get_dependencies_versions()` - JSON-based DEPENDENCIES.json parsing
   - `get_npm_versions()` - shell-out to `npm ls` command
6. **Main Logic**: Parallel processing with rayon, mutex-protected result collection

## Key Patterns

- **Preloading**: Lock files are read once per directory before parallel processing
- **Parallel Processing**: Uses rayon's `par_iter()` with configurable thread pool
- **Result Accumulation**: Thread-safe `Mutex<Vec<>>` for collecting results
- **Exclusions**: Hardcoded exclusion of `node_modules` and `.nx` directories
- **Input Format**: `packages.txt` expects `name@version` format, one per line
- **Output Format**: CSV with columns: package, version, location, match_package, match_version

## File Conventions

- Input file must be named `packages.txt` in project root
- Output file is always `output.csv` in project root
- Backup files use `.backup` extension (e.g., `main.rs.backup`)
