---
inclusion: always
---

# Technology Stack

## Language & Edition

- Rust 2021 edition
- License: MIT OR Apache-2.0

## Core Dependencies

- `clap` (4.0+): CLI argument parsing with derive macros
- `rayon` (1.7): Parallel processing and thread pool management
- `regex` (1.10): Pattern matching for version extraction
- `serde_json` (1.0): JSON parsing for lock files and package.json
- `walkdir` (2.4): Recursive directory traversal
- `csv` (1.3): CSV file generation
- `num_cpus` (1.16): CPU detection for thread pool sizing

## Build System

Standard Cargo workflow:

```bash
# Build for development
cargo build

# Build optimized release binary
cargo build --release

# Run directly
cargo run

# Run with arguments
cargo run -- --verbose --jobs 4

# Run tests (if present)
cargo test
```bash

```

## Code Style Conventions

- Use rustdoc comments (`///` and `//!`) for public APIs
- Struct fields should be documented with `///` comments
- Prefer explicit error handling with `Result` and `Option`
- Use `eprintln!` for debug/error output, `println!` for user-facing output
- Command line args use `clap` derive macros with `#[arg()]` attributes
- Parallel operations use `rayon`'s `par_iter()` pattern
- Mutex-wrapped collections for thread-safe accumulation of results
