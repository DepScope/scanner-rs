# Scanner Documentation

## Build & Release

- **[BUILD_AND_RELEASE.md](BUILD_AND_RELEASE.md)** - Complete guide for building and releasing Scanner
  - Cross-compilation for all platforms
  - Docker-based Linux builds
  - Release process
  - Troubleshooting

## Cross-Compilation Details

- **[CROSS_COMPILE.md](CROSS_COMPILE.md)** - Detailed cross-compilation guide
- **[CROSS_COMPILE_SETUP.md](CROSS_COMPILE_SETUP.md)** - rustup setup instructions
- **[DOCKER_BUILDS.md](DOCKER_BUILDS.md)** - Docker build specifics

## Feature Documentation

- **[SCAN_SUPPLYCHAIN.md](SCAN_SUPPLYCHAIN.md)** - Supply chain scanning
- **[INFECTED_PACKAGE_DETECTION.md](INFECTED_PACKAGE_DETECTION.md)** - Package infection detection
- **[INSTALLED_PACKAGE_ANALYSIS.md](INSTALLED_PACKAGE_ANALYSIS.md)** - Package analysis features
- **[IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)** - Implementation details

## Development

- **[PRE_COMMIT_HOOKS.md](PRE_COMMIT_HOOKS.md)** - Pre-commit hook setup
- **[PRE_COMMIT_QUICK_REFERENCE.md](PRE_COMMIT_QUICK_REFERENCE.md)** - Quick reference
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions

## Research

- **[research.md](research.md)** - Research notes and findings

## Quick Start

### Build All Platforms

```bash
make cross-compile
```

### Create Release

```bash
make release-patch
```

### Get Help

```bash
make help
```

For detailed instructions, start with [BUILD_AND_RELEASE.md](BUILD_AND_RELEASE.md).
