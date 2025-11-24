---
inclusion: always
---

# Product Overview

Scanner is a CLI tool for analyzing npm package compatibility across JavaScript/TypeScript projects. It recursively scans directories to identify npm packages and verify their presence across different package manager lock files (yarn.lock, package-lock.json, pnpm-lock.yaml) and dependency manifests.

## Primary Use Cases

- Auditing monorepos for package version consistency
- Identifying which packages are installed vs declared
- Cross-referencing package versions across multiple lock file formats
- Generating CSV reports of package distribution across a codebase

## Key Capabilities

- Multi-package manager support (npm, yarn, pnpm)
- Parallel directory scanning with configurable thread count
- CSV export for analysis and reporting
- Recursive directory traversal with smart exclusions (node_modules, .nx)
