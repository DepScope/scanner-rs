# release-all.sh - Unified Release Script

One script to rule them all! This script combines checking, building, cross-compiling, and releasing into a single command.

## Quick Start

```bash
# Create a patch release (0.1.0 → 0.1.1)
./release-all.sh --patch

# Create a minor release (0.1.0 → 0.2.0)
./release-all.sh --minor

# Create a major release (0.1.0 → 1.0.0)
./release-all.sh --major

# Test without making changes
./release-all.sh --patch --dry-run
```

## What It Does

The script performs 5 steps:

### [1/5] Pre-flight Checks
- ✓ Git repository exists
- ✓ No uncommitted changes
- ✓ GitHub CLI installed and authenticated
- ✓ Can access GitHub repository
- ✓ Rust toolchain available
- ✓ Checks rustup for cross-compilation capability

### [2/5] Update Version
- Updates version in `Cargo.toml`
- Updates `Cargo.lock`
- Commits changes
- Creates git tag
- Pushes to remote

### [3/5] Build Binaries
- Builds native binary for your platform
- **If rustup is configured**: Cross-compiles for other architectures
  - macOS: Builds for both x86_64 and aarch64
  - Linux: Builds for both x86_64 and aarch64 (requires `cross` tool)
- **If rustup is not configured**: Only builds native binary

### [4/5] Build Summary
- Lists all created binaries
- Shows file sizes and architectures

### [5/5] Create GitHub Release
- Creates GitHub release with all binaries
- Generates installation instructions
- Uploads all binaries

## Cross-Compilation

### Current Setup (Homebrew Rust)
- ✅ Works perfectly for native builds
- ⚠️ Cannot cross-compile (limited to 1 binary)

### With rustup Configured
- ✅ Builds for multiple architectures
- ✅ Creates 2-3 binaries per release

To enable cross-compilation:
```bash
# Set up rustup
rustup default stable

# Add targets for macOS
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Now release-all.sh will build all architectures
./release-all.sh --patch
```

## Output

The script creates binaries in `target/release-artifacts/`:
- `scanner-X.Y.Z-darwin-aarch64` (Apple Silicon)
- `scanner-X.Y.Z-darwin-x86_64` (Intel Mac) - if rustup configured
- `scanner-X.Y.Z-linux-x86_64` (Linux) - if on Linux with cross-compilation

All binaries are automatically uploaded to the GitHub release.

## Examples

### Successful Release (with rustup)
```
╔════════════════════════════════════════╗
║   Scanner Multi-Platform Release      ║
╚════════════════════════════════════════╝

[1/5] Pre-flight Checks
  ✓ Git repository found
  ✓ No uncommitted changes
  ✓ GitHub CLI installed
  ✓ GitHub CLI authenticated
  ✓ Repository: DepScope/scanner-rs
  ✓ Rust toolchain installed
  ✓ Current version: 0.1.2
  ✓ rustup configured (cross-compilation enabled)

  Version bump: 0.1.2 → 0.1.3

Continue with release? (y/N) y

[2/5] Updating Version
  ✓ Updated Cargo.toml
  ✓ Updated Cargo.lock
  ✓ Committed version bump
  ✓ Created tag v0.1.3
  ✓ Pushed to remote

[3/5] Building Binaries
  ✓ Built scanner-0.1.3-darwin-aarch64 (native)
  
  Cross-compiling for macOS...
  → Building for x86_64-apple-darwin...
  ✓ Built scanner-0.1.3-darwin-x86_64

[4/5] Build Summary
  ✓ Created 2 binaries:
    scanner-0.1.3-darwin-aarch64
      Size: 3.5M, Arch: arm64
    scanner-0.1.3-darwin-x86_64
      Size: 3.8M, Arch: x86_64

[5/5] Creating GitHub Release
  ✓ GitHub release created

╔════════════════════════════════════════╗
║         Release Complete! 🎉           ║
╚════════════════════════════════════════╝

  Version: v0.1.3
  Binaries: 2
  Release: https://github.com/DepScope/scanner-rs/releases/tag/v0.1.3
```

### Release without rustup
Same output but only 1 binary (native architecture).

## Troubleshooting

### "You have uncommitted changes"
Commit or stash your changes first:
```bash
git status
git add .
git commit -m "your message"
```

### "GitHub CLI not authenticated"
```bash
gh auth login
```

### "rustup found but no active toolchain"
```bash
rustup default stable
```

### Release failed
Delete the tag and try again:
```bash
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z
gh release delete vX.Y.Z --yes
```

## Comparison with Other Scripts

| Script | Purpose | When to Use |
|--------|---------|-------------|
| `release-all.sh` | **All-in-one** release | **Recommended** for releases |
| `release.sh` | Original release script | Legacy, use release-all.sh instead |
| `check-release.sh` | Pre-flight checks only | Testing readiness |
| `cross-compile.sh` | Test cross-compilation | Testing build setup |

## Makefile Shortcuts

```bash
make release-patch    # Same as ./release-all.sh --patch
make release-minor    # Same as ./release-all.sh --minor
make release-major    # Same as ./release-all.sh --major
make release-dry-run  # Same as ./release-all.sh --patch --dry-run
```
