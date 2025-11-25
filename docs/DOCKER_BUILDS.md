# Docker-Based Linux Builds

Scanner supports building Linux binaries using Docker, which solves cross-compilation issues when building from macOS.

## Why Docker?

Cross-compiling Linux binaries from macOS can be complex due to:

- Missing system libraries
- Linker incompatibilities
- Different libc versions

Docker provides a native Linux environment for building, ensuring compatibility.

## Prerequisites

### Install Docker Desktop

Download and install Docker Desktop for Mac:
<https://docs.docker.com/desktop/install/mac-install/>

After installation:

1. Start Docker Desktop
2. Verify it's running: `docker --version`

## Usage

### Automatic (Recommended)

The `cross-compile.sh` script automatically uses Docker for Linux builds if available:

```bash
# Will use Docker for Linux targets automatically
./cross-compile.sh

# Or via Makefile
make cross-compile
```

**Behavior:**

1. Tries native cross-compilation first
2. If that fails for Linux targets, automatically falls back to Docker
3. Builds other platforms (macOS, Windows) natively

### Manual Linux-Only Builds

Build only Linux binaries using Docker:

```bash
# Build both Linux AMD64 and x86
./docker-build-linux.sh

# Or via Makefile
make docker-build-linux
```

## What Gets Built

The Docker build creates:

### Linux AMD64 (x86_64)

- **Target:** `x86_64-unknown-linux-gnu`
- **Binary:** `target/x86_64-unknown-linux-gnu/release/scanner`
- **Archive:** `dist/scanner-v0.2.2-linux-amd64.tar.gz`
- **Platform:** `linux/amd64`

### Linux x86 (32-bit)

- **Target:** `i686-unknown-linux-gnu`
- **Binary:** `target/i686-unknown-linux-gnu/release/scanner`
- **Archive:** `dist/scanner-v0.2.2-linux-x86.tar.gz`
- **Platform:** `linux/386`

## How It Works

The Docker build:

1. **Pulls Rust image:** Uses official `rust:latest` Docker image
2. **Mounts workspace:** Your project directory is mounted into the container
3. **Installs target:** Runs `rustup target add <target>` inside container
4. **Builds:** Runs `cargo build --release --target <target>`
5. **Outputs:** Binaries are written to your local `target/` directory

## Build Time

**First build:**

- Downloads Rust Docker image (~1.5GB)
- Downloads and compiles dependencies
- Takes 5-10 minutes

**Subsequent builds:**

- Uses cached image and dependencies
- Takes 1-2 minutes

## Troubleshooting

### "Cannot connect to Docker daemon"

Docker Desktop isn't running. Start it from Applications.

### "docker: command not found"

Docker isn't installed. Download from:
<https://docs.docker.com/desktop/install/mac-install/>

### Builds are slow

First build downloads the Rust image and compiles dependencies. Subsequent builds are much faster due to caching.

### Permission errors

The Docker container runs as root, so built files may have root ownership. This is normal and doesn't affect usage.

To fix ownership:

```bash
sudo chown -R $(whoami) target/
```

### Out of disk space

Docker images can be large. Clean up unused images:

```bash
docker system prune -a
```

## Verifying Binaries

Check the built Linux binaries:

```bash
# Check architecture
file target/x86_64-unknown-linux-gnu/release/scanner
# Output: ELF 64-bit LSB executable, x86-64

file target/i686-unknown-linux-gnu/release/scanner
# Output: ELF 32-bit LSB executable, Intel 80386

# Check they're Linux binaries
file target/x86_64-unknown-linux-gnu/release/scanner | grep Linux
```

## Testing on Linux

To test the binaries on Linux, you can use Docker:

```bash
# Test AMD64 binary
docker run --rm -v "$PWD:/app" -w /app ubuntu:latest \
  /app/target/x86_64-unknown-linux-gnu/release/scanner --version

# Test x86 binary
docker run --rm --platform linux/386 -v "$PWD:/app" -w /app ubuntu:latest \
  /app/target/i686-unknown-linux-gnu/release/scanner --version
```

## Integration with Release Process

The `release-all.sh` script doesn't automatically use Docker (to avoid unexpected delays). To include Linux binaries in releases:

### Option 1: Build Linux binaries first

```bash
# Build Linux binaries with Docker
make docker-build-linux

# Then create release (will include Linux binaries)
make release-patch
```

### Option 2: Use cross-compile script

```bash
# This will auto-use Docker for Linux
./cross-compile.sh

# Then create release
make release-patch
```

### Option 3: Manual workflow

```bash
# Build everything
./cross-compile.sh  # Builds macOS, Windows, and Linux (via Docker)

# Create release with all binaries
./release-all.sh --patch
```

## Docker Image Details

**Image:** `rust:latest`

- Based on Debian
- Includes rustc, cargo, rustup
- Updated regularly
- Size: ~1.5GB

**Platforms supported:**

- `linux/amd64` - For x86_64 builds
- `linux/386` - For i686 builds

## Alternative: Native Linux Build

If you have access to a Linux machine, you can build natively without Docker:

```bash
# On Linux
./cross-compile.sh  # Will build all targets natively
```

This is faster and doesn't require Docker.

## Summary

| Method | Platforms | Speed | Requirements |
|--------|-----------|-------|--------------|
| Native cross-compile | macOS, Windows | Fast | rustup, mingw-w64 |
| Docker (auto) | All 5 platforms | Medium | Docker Desktop |
| Docker (manual) | Linux only | Medium | Docker Desktop |
| Native Linux | All 5 platforms | Fastest | Linux machine |

## See Also

- [CROSS_COMPILE.md](./CROSS_COMPILE.md) - General cross-compilation guide
- [CROSS_COMPILE_SETUP.md](./CROSS_COMPILE_SETUP.md) - rustup setup
- [Docker Desktop Documentation](https://docs.docker.com/desktop/)
