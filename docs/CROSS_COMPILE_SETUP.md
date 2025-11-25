# Cross-Compilation Setup Guide

## Current Situation

You have Rust installed via **Homebrew**, which doesn't support cross-compilation targets. To build binaries for multiple architectures (e.g., both Intel and Apple Silicon Macs), you need **rustup**.

## Why Cross-Compile?

Cross-compilation allows you to build binaries for different architectures from a single machine:

- **Apple Silicon (M1/M2)** → Build for Intel Macs
- **Intel Mac** → Build for Apple Silicon
- **Linux x86_64** → Build for ARM64

This means you can create releases with binaries for all platforms without needing multiple machines.

## Setup Instructions

### Option 1: Install rustup (Recommended)

This gives you full control over Rust toolchains and cross-compilation targets.

```bash
# 1. Install rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Follow the prompts (default options are fine)

# 3. Restart your shell or run:
source $HOME/.cargo/env

# 4. Add cross-compilation targets for macOS
rustup target add x86_64-apple-darwin    # Intel Mac
rustup target add aarch64-apple-darwin   # Apple Silicon

# 5. Verify installation
rustup show
rustup target list --installed
```

### Option 2: Keep Homebrew Rust (Limited)

If you prefer to keep Homebrew's Rust, you can only build for your native architecture. The release script will work but only create binaries for your current platform.

## Testing Cross-Compilation

After installing rustup, test cross-compilation:

```bash
./cross-compile.sh
```

This will:

- Check if rustup is installed
- Install missing targets
- Build for all supported architectures
- Show binary sizes and info

## Building Releases

### With rustup (Full Cross-Compilation)

```bash
# This will build for both x86_64 and aarch64
./release.sh --patch
```

### Without rustup (Native Only)

```bash
# This will only build for your current architecture
./release.sh --patch --no-cross-compile
```

Or the script will automatically skip cross-compilation if rustup isn't available.

## Verifying Binaries

After building, check what you have:

```bash
# List all built binaries
find target -name scanner -type f -path "*/release/*" -exec ls -lh {} \;

# Check architecture of a binary
file target/release/scanner
file target/x86_64-apple-darwin/release/scanner
file target/aarch64-apple-darwin/release/scanner
```

## Troubleshooting

### "rustup: command not found" after installation

Restart your terminal or run:

```bash
source $HOME/.cargo/env
```

### Homebrew and rustup conflict

If you have both, rustup should take precedence. Check your PATH:

```bash
which cargo
which rustc
```

Should show paths like `~/.cargo/bin/cargo` (rustup) not `/opt/homebrew/bin/cargo` (Homebrew).

To remove Homebrew's Rust:

```bash
brew uninstall rust
```

### Cross-compilation fails

Make sure you have Xcode Command Line Tools:

```bash
xcode-select --install
```

## Linux Cross-Compilation

For Linux, you need the `cross` tool:

```bash
cargo install cross

# Then you can build for different Linux architectures
cross build --release --target x86_64-unknown-linux-gnu
cross build --release --target aarch64-unknown-linux-gnu
```

## Summary

| Setup | Native Build | Cross-Compile | Recommendation |
|-------|-------------|---------------|----------------|
| Homebrew Rust | ✅ | ❌ | Use for quick local dev |
| rustup | ✅ | ✅ | Use for releases |

For creating releases with multiple architectures, **rustup is required**.
