# Release Process

This document describes how to create a new release of Scanner.

## Prerequisites

1. **GitHub CLI**: Install from https://cli.github.com/
   ```bash
   brew install gh  # macOS
   ```

2. **Authentication**: Ensure you're authenticated with GitHub
   ```bash
   gh auth login
   ```

3. **Clean working directory**: Commit or stash all changes
   ```bash
   git status
   ```

4. **Cross-compilation (Optional)**: For building multiple architectures
   - See [CROSS_COMPILE_SETUP.md](CROSS_COMPILE_SETUP.md) for detailed instructions
   - Without rustup: Only native architecture will be built
   - With rustup: All architectures will be built

## Local Release (Manual)

Use the `release.sh` script to create a release:

### Patch Release (0.1.0 → 0.1.1)
```bash
./release.sh --patch
```

### Minor Release (0.1.0 → 0.2.0)
```bash
./release.sh --minor
```

### Major Release (0.1.0 → 1.0.0)
```bash
./release.sh --major
```

### Dry Run
Test the release process without making changes:
```bash
./release.sh --patch --dry-run
```

## What the Script Does

1. **Validates environment**
   - Checks for git repository
   - Verifies no uncommitted changes
   - Confirms GitHub CLI is installed

2. **Updates version**
   - Bumps version in `Cargo.toml`
   - Updates `Cargo.lock`
   - Commits changes

3. **Creates git tag**
   - Tags commit with new version
   - Pushes tag to remote

4. **Builds binaries**
   - Compiles release binary for current architecture
   - Attempts cross-compilation (macOS only)
   - Creates architecture-specific binaries

5. **Creates GitHub release**
   - Uploads all binaries
   - Generates release notes
   - Publishes release

## Automated Release (CI/CD)

When you push a tag, GitHub Actions automatically:

1. Builds binaries for multiple platforms:
   - Linux x86_64
   - Linux aarch64
   - macOS x86_64 (Intel)
   - macOS aarch64 (Apple Silicon)

2. Creates a GitHub release with all binaries

### Manual trigger
```bash
git tag v0.2.0
git push origin v0.2.0
```

## Binary Naming Convention

Binaries follow this pattern:
```
scanner-{version}-{os}-{arch}
```

Examples:
- `scanner-0.1.0-darwin-aarch64` (macOS Apple Silicon)
- `scanner-0.1.0-darwin-x86_64` (macOS Intel)
- `scanner-0.1.0-linux-x86_64` (Linux)

## Troubleshooting

### Cross-compilation fails on macOS
This is expected if you don't have the cross-compilation toolchain. The script will continue with the native binary only.

To enable cross-compilation:
```bash
rustup target add aarch64-apple-darwin  # If on Intel Mac
rustup target add x86_64-apple-darwin   # If on Apple Silicon
```

### GitHub CLI authentication issues
```bash
gh auth status
gh auth login
```

### Release already exists
Delete the tag and release, then try again:
```bash
git tag -d v0.1.0
git push origin :refs/tags/v0.1.0
gh release delete v0.1.0
```

## Version Strategy

Follow semantic versioning (semver):

- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)
