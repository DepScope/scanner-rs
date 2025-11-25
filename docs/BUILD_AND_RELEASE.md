# Scanner - Build and Release Guide

Complete guide for building Scanner across multiple platforms and creating releases.

## Quick Start

```bash
# Build all platforms (macOS, Linux, Windows)
make cross-compile

# Create a new release
make release-patch
```

## Supported Platforms

Scanner builds for 5 platforms from macOS:

| Platform | Architecture | Method | Requirements |
|----------|-------------|--------|--------------|
| macOS | ARM64 (Apple Silicon) | Native | None |
| macOS | AMD64 (Intel) | Cross-compile | rustup |
| Linux | AMD64 (x86_64) | Docker | Docker Desktop |
| Linux | x86 (32-bit) | Docker | Docker Desktop |
| Windows | AMD64 | Cross-compile | mingw-w64 |

## Prerequisites

### Required

**rustup** (not Homebrew's Rust):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Optional (for all platforms)

**Docker Desktop** (for Linux builds):

```bash
# Download from: https://docs.docker.com/desktop/install/mac-install/
# Or install via Homebrew:
brew install --cask docker
```

**MinGW** (for Windows builds):

```bash
brew install mingw-w64
```

## Building

### Build All Platforms

```bash
# Automatic - uses Docker for Linux if available
make cross-compile

# Or directly
./cross-compile.sh
```

**Output:**

```
dist/
├── scanner-v0.2.2-macos-arm64.tar.gz
├── scanner-v0.2.2-macos-amd64.tar.gz
├── scanner-v0.2.2-linux-amd64.tar.gz
├── scanner-v0.2.2-linux-x86.tar.gz
└── scanner-v0.2.2-windows-amd64.zip
```

### Build Linux Only

```bash
# Docker-based Linux builds
make docker-build-linux

# Or directly
./docker-build-linux.sh
```

### Build Native Only

```bash
# Just your current platform
cargo build --release
```

## How It Works

### Cross-Compilation Strategy

**macOS builds:**

- ARM64: Native compilation
- AMD64: Cross-compile using clang

**Windows builds:**

- Cross-compile using MinGW toolchain
- Requires `mingw-w64` package

**Linux builds:**

- Attempts native cross-compilation first
- Falls back to Docker if native fails
- Uses `rust:latest` Docker image
- Platform-specific: `linux/amd64` and `linux/386`

### Build Process Flow

```
cross-compile.sh / release-all.sh
├── Check rustup and Docker availability
├── For each target:
│   ├── Install target if needed
│   ├── Try native cross-compilation
│   ├── If Linux target fails:
│   │   ├── Check Docker available
│   │   ├── Attempt Docker build
│   │   └── Copy binary if successful
│   └── Continue to next target
└── Create distribution archives
```

## Creating Releases

### Release Process

```bash
# Patch release (0.1.0 → 0.1.1)
make release-patch

# Minor release (0.1.0 → 0.2.0)
make release-minor

# Major release (0.1.0 → 1.0.0)
make release-major
```

### What Happens

1. **Pre-flight checks** - Git status, GitHub CLI, rustup
2. **Version bump** - Updates Cargo.toml and Cargo.lock
3. **Build binaries** - All platforms (with Docker fallback)
4. **Create tag** - Git tag with version
5. **GitHub release** - Creates release and uploads binaries

### Dry Run

Test the release process without making changes:

```bash
./release-all.sh --patch --dry-run
```

### Reupload Binaries

If you need to reupload binaries for an existing release:

```bash
./release-all.sh --patch --reupload
```

## Build Times

| Build Type | First Time | Cached |
|------------|-----------|--------|
| macOS (native) | 30s | 10s |
| macOS (cross) | 45s | 15s |
| Windows | 60s | 20s |
| Linux (Docker) | 10min | 2min |

**Note:** First Docker build downloads ~1.5GB Rust image.

## Troubleshooting

### Cargo not found

The scripts automatically add rustup's cargo to PATH. If issues persist:

```bash
# Add to ~/.zshrc
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.zshrc
```

### Homebrew Rust conflict

Remove Homebrew's Rust to use rustup:

```bash
brew uninstall rust
```

Or adjust PATH priority:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

### Docker not available

Install Docker Desktop:

```bash
brew install --cask docker
```

Start Docker Desktop and verify:

```bash
docker info
```

### Windows build fails

Install MinGW:

```bash
brew install mingw-w64
```

### Linux build fails (without Docker)

This is expected on macOS without Docker. Options:

1. Install Docker Desktop
2. Build on Linux natively
3. Use GitHub Actions (CI/CD)

### Bash version errors

Scripts require Bash 4+. macOS includes Bash 3.2 by default.

Install modern Bash:

```bash
brew install bash
```

Scripts use `#!/usr/bin/env bash` which finds the newer version.

### Permission errors (Docker)

Docker runs as root. Fix ownership:

```bash
sudo chown -R $(whoami) target/
```

## Verifying Binaries

### Check Built Binaries

```bash
# List all binaries
ls -lh target/*/release/scanner*

# Check architectures
file target/aarch64-apple-darwin/release/scanner
file target/x86_64-apple-darwin/release/scanner
file target/x86_64-unknown-linux-gnu/release/scanner
file target/i686-unknown-linux-gnu/release/scanner
file target/x86_64-pc-windows-gnu/release/scanner.exe
```

### Expected Output

```
Mach-O 64-bit executable arm64
Mach-O 64-bit executable x86_64
ELF 64-bit LSB executable, x86-64
ELF 32-bit LSB executable, Intel 80386
PE32+ executable (console) x86-64
```

### Test Binaries

```bash
# Test native binary
./target/release/scanner --version

# Test Linux binary in Docker
docker run --rm -v "$PWD:/app" -w /app ubuntu:latest \
  /app/target/x86_64-unknown-linux-gnu/release/scanner --version
```

## Configuration

### Cargo Configuration

`.cargo/config.toml` contains linker settings:

```toml
[target.x86_64-apple-darwin]
linker = "clang"
rustflags = ["-C", "link-arg=-mmacosx-version-min=10.7"]

[target.x86_64-pc-windows-gnu]
linker = "x86_64-w64-mingw32-gcc"
```

### Rust Targets

Installed automatically by scripts:

```bash
# View installed targets
rustup target list --installed

# Manually add target
rustup target add <target-name>
```

## Scripts Reference

### cross-compile.sh

Main cross-compilation script with Docker fallback.

**Features:**

- Builds all 5 platforms
- Automatic Docker fallback for Linux
- Creates distribution archives
- Helpful error messages

**Usage:**

```bash
./cross-compile.sh
```

### docker-build-linux.sh

Linux-only Docker builds.

**Features:**

- Builds Linux AMD64 and x86
- Uses Docker exclusively
- Faster if you only need Linux

**Usage:**

```bash
./docker-build-linux.sh
```

### release-all.sh

Complete release automation.

**Features:**

- Version management
- Cross-compilation with Docker fallback
- Git tagging
- GitHub release creation
- Binary uploads

**Usage:**

```bash
./release-all.sh [--patch|--minor|--major] [--dry-run] [--reupload]
```

## Makefile Targets

```bash
# Development
make build          # Build release binary
make test           # Run tests
make check          # Check compilation
make fmt            # Format code
make clippy         # Run linter

# Cross-compilation
make cross-compile      # Build all platforms
make docker-build-linux # Build Linux only

# Releases
make release-patch  # Patch release
make release-minor  # Minor release
make release-major  # Major release

# Help
make help          # Show all targets
```

## GitHub Actions (Optional)

For automated releases on native runners, enable the workflow:

```bash
mv .github/workflows/release.yml.off .github/workflows/release.yml
```

This builds on native runners for each platform:

- macOS builds on `macos-latest`
- Linux builds on `ubuntu-latest`
- Windows builds on `windows-latest`

## Best Practices

1. **Test locally first** - Use `--dry-run` before actual releases
2. **Verify binaries** - Check architectures with `file` command
3. **Clean builds** - Run `cargo clean` if issues occur
4. **Docker cache** - First build is slow, subsequent builds are fast
5. **Commit before release** - Ensure clean git state

## Distribution

Users can download binaries from GitHub Releases:

### macOS

```bash
# Apple Silicon
curl -L https://github.com/USER/REPO/releases/download/v0.2.2/scanner-v0.2.2-macos-arm64.tar.gz | tar xz
sudo mv scanner /usr/local/bin/

# Intel
curl -L https://github.com/USER/REPO/releases/download/v0.2.2/scanner-v0.2.2-macos-amd64.tar.gz | tar xz
sudo mv scanner /usr/local/bin/
```

### Linux

```bash
# AMD64
curl -L https://github.com/USER/REPO/releases/download/v0.2.2/scanner-v0.2.2-linux-amd64.tar.gz | tar xz
sudo mv scanner /usr/local/bin/

# x86 (32-bit)
curl -L https://github.com/USER/REPO/releases/download/v0.2.2/scanner-v0.2.2-linux-x86.tar.gz | tar xz
sudo mv scanner /usr/local/bin/
```

### Windows

```powershell
# Download and extract
Invoke-WebRequest -Uri https://github.com/USER/REPO/releases/download/v0.2.2/scanner-v0.2.2-windows-amd64.zip -OutFile scanner.zip
Expand-Archive scanner.zip
.\scanner\scanner.exe --help
```

## Summary

- ✅ **5 platforms** supported from macOS
- ✅ **Automatic Docker fallback** for Linux
- ✅ **Single command** builds all platforms
- ✅ **Integrated release** process
- ✅ **Distribution archives** created automatically

For more details, see:

- `docs/CROSS_COMPILE.md` - Cross-compilation details
- `docs/DOCKER_BUILDS.md` - Docker build specifics
- `docs/CROSS_COMPILE_SETUP.md` - rustup setup guide
