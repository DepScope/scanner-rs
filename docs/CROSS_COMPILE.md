# Cross-Compilation Guide

Scanner supports cross-compilation for multiple platforms from a single build machine.

## Supported Platforms

- **macOS ARM64** (Apple Silicon: M1/M2/M3/M4)
- **macOS AMD64** (Intel)
- **Linux AMD64** (x86_64)
- **Linux x86** (32-bit i686)
- **Windows AMD64** (x86_64)

## Quick Start

### Using the Release Script (Recommended)

The `release-all.sh` script automatically handles cross-compilation:

```bash
# Create a new release with all platform binaries
make release-patch

# Or directly
./release-all.sh --patch
```

This will:

1. Build binaries for all supported platforms
2. Create a GitHub release
3. Upload all binaries

### Manual Cross-Compilation

Use the standalone cross-compilation script:

```bash
# Build for all platforms
make cross-compile

# Or directly
./cross-compile.sh
```

Binaries will be created in the `dist/` directory with platform-specific archives.

## Requirements

### rustup (Required)

Cross-compilation requires `rustup` instead of Homebrew's Rust:

```bash
# Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Restart your shell or run
source $HOME/.cargo/env

# Verify
rustup --version
```

### Targets

The scripts automatically install required targets:

- `aarch64-apple-darwin` - macOS ARM64
- `x86_64-apple-darwin` - macOS AMD64
- `x86_64-unknown-linux-gnu` - Linux AMD64
- `i686-unknown-linux-gnu` - Linux x86 (32-bit)
- `x86_64-pc-windows-gnu` - Windows AMD64

Manual installation:

```bash
rustup target add aarch64-apple-darwin
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
```

## Build Output

### Release Script

Binaries are placed in `target/release-artifacts/`:

```
scanner-0.2.2-darwin-arm64
scanner-0.2.2-darwin-amd64
scanner-0.2.2-linux-amd64
scanner-0.2.2-linux-x86
scanner-0.2.2-windows-amd64.exe
```

### Cross-Compile Script

Archives are created in `dist/`:

```
scanner-v0.2.2-macos-arm64.tar.gz
scanner-v0.2.2-macos-amd64.tar.gz
scanner-v0.2.2-linux-amd64.tar.gz
scanner-v0.2.2-linux-x86.tar.gz
scanner-v0.2.2-windows-amd64.zip
```

## Verifying Binaries

Check the architecture of built binaries:

```bash
# macOS/Linux
file target/aarch64-apple-darwin/release/scanner
file target/x86_64-apple-darwin/release/scanner
file target/x86_64-unknown-linux-gnu/release/scanner
file target/i686-unknown-linux-gnu/release/scanner
file target/x86_64-pc-windows-gnu/release/scanner.exe

# Expected output examples:
# Mach-O 64-bit executable arm64
# Mach-O 64-bit executable x86_64
# ELF 64-bit LSB executable, x86-64
# ELF 32-bit LSB executable, Intel 80386
# PE32+ executable (console) x86-64
```

## Troubleshooting

### "rustup: command not found"

Restart your terminal or run:

```bash
source $HOME/.cargo/env
```

### Windows Build Fails

The Windows target (`x86_64-pc-windows-gnu`) requires MinGW. On macOS:

```bash
brew install mingw-w64
```

If builds still fail, the release script will continue with other platforms.

### Cross-Compilation Not Available

If you see this warning, you're using Homebrew's Rust instead of rustup:

```
⚠ Cross-compilation not available
  Install rustup to enable: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The script will only build for your native platform.

### Checking Active Toolchain

```bash
# Show active toolchain
rustup show

# Should show something like:
# Default host: aarch64-apple-darwin
# rustup home:  /Users/you/.rustup
#
# stable-aarch64-apple-darwin (default)
# rustc 1.75.0
```

## Platform-Specific Notes

### macOS

- Can build for both ARM64 and AMD64 from either architecture
- No additional tools required beyond rustup

### Linux

- Building from macOS works for basic binaries
- For complex dependencies, consider using Docker or `cross` tool

### Windows

- Requires MinGW toolchain on macOS/Linux
- Produces `.exe` files
- May have limitations with some system dependencies

## CI/CD Integration

The GitHub Actions workflow (`.github/workflows/release.yml.off`) can be enabled for automated releases. It builds on native runners for each platform:

- macOS builds on `macos-latest`
- Linux builds on `ubuntu-latest`
- Windows builds on `windows-latest`

This ensures maximum compatibility but requires GitHub Actions minutes.

## Best Practices

1. **Test on Target Platform**: Cross-compiled binaries should be tested on their target platforms
2. **Use Release Script**: The `release-all.sh` script handles version bumping, tagging, and uploading
3. **Check Binary Sizes**: Cross-compiled binaries should be similar in size to native builds
4. **Verify Architecture**: Always verify with `file` command before releasing

## See Also

- [CROSS_COMPILE_SETUP.md](./CROSS_COMPILE_SETUP.md) - Detailed rustup setup guide
- [Rust Platform Support](https://doc.rust-lang.org/nightly/rustc/platform-support.html)
- [rustup Documentation](https://rust-lang.github.io/rustup/)
