# Release Guide

Complete guide for creating releases of Scanner with multi-platform binaries.

## Quick Start

```bash
# Create a release (one command does everything!)
./release-all.sh --patch   # 0.1.0 → 0.1.1
./release-all.sh --minor   # 0.1.0 → 0.2.0
./release-all.sh --major   # 0.1.0 → 1.0.0

# Or use make
make release-patch
make release-minor
make release-major

# Test without making changes
./release-all.sh --patch --dry-run
```

## Prerequisites

1. **GitHub CLI** - Install and authenticate:
   ```bash
   brew install gh
   gh auth login
   ```

2. **Clean working directory** - Commit all changes:
   ```bash
   git status
   ```

3. **Rust toolchain** - Already installed ✓

4. **Cross-compilation (Optional)** - For multiple binaries:
   ```bash
   rustup default stable
   rustup target add x86_64-apple-darwin aarch64-apple-darwin
   ```

## What the Script Does

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
- Commits changes with message: `chore: bump version to X.Y.Z`
- Creates git tag `vX.Y.Z`
- Pushes to remote

### [3/5] Build Binaries
**Without rustup configured:**
- Builds 1 binary for your native architecture

**With rustup configured:**
- Builds native binary
- Cross-compiles for other architectures:
  - macOS: x86_64 (Intel) + aarch64 (Apple Silicon)
  - Linux: x86_64 + aarch64 (requires `cross` tool)

### [4/5] Build Summary
- Lists all created binaries
- Shows file sizes and architectures

### [5/5] Create GitHub Release
- Creates GitHub release with tag
- Uploads all binaries
- Generates installation instructions

## Binary Output

Binaries are created in `target/release-artifacts/`:

| Binary | Platform | When Created |
|--------|----------|--------------|
| `scanner-X.Y.Z-darwin-aarch64` | macOS Apple Silicon | Always (if on macOS ARM) |
| `scanner-X.Y.Z-darwin-x86_64` | macOS Intel | With rustup |
| `scanner-X.Y.Z-linux-x86_64` | Linux x86_64 | Always (if on Linux) |
| `scanner-X.Y.Z-linux-aarch64` | Linux ARM64 | With cross tool |

## Cross-Compilation Setup

### Current Setup
You have rustup installed but not configured. This means:
- ✅ Can create releases
- ⚠️ Only 1 binary per release (native architecture)

### Enable Cross-Compilation

To build binaries for multiple architectures:

```bash
# 1. Configure rustup
rustup default stable

# 2. Add targets for macOS
rustup target add x86_64-apple-darwin    # Intel Mac
rustup target add aarch64-apple-darwin   # Apple Silicon

# 3. Verify
rustup show
rustup target list --installed

# 4. Test
./release-all.sh --patch --dry-run
```

### Why Cross-Compile?

**Without cross-compilation:**
- 1 binary per release
- Users on other architectures must build from source

**With cross-compilation:**
- 2-3 binaries per release
- Users can download pre-built binaries for their platform
- Better user experience

## Example Output

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

## Troubleshooting

### "You have uncommitted changes"
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

### "Cannot access GitHub repository"
```bash
# Check remote
git remote -v

# Verify gh can access it
gh repo view
```

### Release failed
Delete the tag and try again:
```bash
# Delete release
gh release delete vX.Y.Z --yes

# Delete tag locally
git tag -d vX.Y.Z

# Delete tag remotely
git push origin :refs/tags/vX.Y.Z

# Try again
./release-all.sh --patch
```

### Cross-compilation fails
This is usually fine - the script will continue with native binary only.

To fix:
```bash
# Make sure targets are installed
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Check Xcode Command Line Tools (macOS)
xcode-select --install
```

## Version Strategy

Follow semantic versioning:

- **MAJOR** (--major): Breaking changes (1.0.0 → 2.0.0)
- **MINOR** (--minor): New features, backward compatible (1.0.0 → 1.1.0)
- **PATCH** (--patch): Bug fixes, backward compatible (1.0.0 → 1.0.1)

## Manual Release (If Script Fails)

If the automated script fails, you can release manually:

```bash
# 1. Update version
sed -i '' 's/version = "0.1.0"/version = "0.1.1"/' Cargo.toml

# 2. Build
cargo build --release

# 3. Commit
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 0.1.1"

# 4. Tag
git tag -a v0.1.1 -m "Release v0.1.1"

# 5. Push
git push origin main
git push origin v0.1.1

# 6. Create release directory
mkdir -p target/release-artifacts
cp target/release/scanner target/release-artifacts/scanner-0.1.1-darwin-aarch64

# 7. Create GitHub release
gh release create v0.1.1 \
  --title "v0.1.1" \
  --notes "Release v0.1.1" \
  target/release-artifacts/*
```

## Files

| File | Purpose |
|------|---------|
| `release-all.sh` | Main release script (use this!) |
| `RELEASING.md` | This file - complete documentation |
| `TROUBLESHOOTING.md` | Detailed troubleshooting guide |
| `CROSS_COMPILE_SETUP.md` | Cross-compilation setup guide |
| `Makefile` | Convenient shortcuts |
| `.github/workflows/release.yml` | CI/CD (optional) |

## Next Steps

1. **Try a dry run:**
   ```bash
   ./release-all.sh --patch --dry-run
   ```

2. **Enable cross-compilation (optional):**
   ```bash
   rustup default stable
   rustup target add x86_64-apple-darwin aarch64-apple-darwin
   ```

3. **Create your first release:**
   ```bash
   ./release-all.sh --patch
   ```

## Getting Help

- Check `TROUBLESHOOTING.md` for common issues
- Check `CROSS_COMPILE_SETUP.md` for cross-compilation help
- Run `./release-all.sh --dry-run` to test without changes
- Check GitHub Actions logs (if using CI/CD)
