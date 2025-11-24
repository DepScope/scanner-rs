# Release Troubleshooting Guide

## Common Issues and Solutions

### Issue: "Cargo.lock is ignored by .gitignore"

**Problem**: Git refuses to add `Cargo.lock` because it's in `.gitignore`.

**Solution**:

```bash
# Remove Cargo.lock from .gitignore (already done in latest version)
# Or force add it once
git add -f Cargo.lock
git commit -m "chore: track Cargo.lock for binary project"
```bash

```

**Why**: Binary projects (CLI tools) should commit `Cargo.lock` to ensure reproducible builds. Library projects typically ignore it.

---

### Issue: "GitHub CLI is not authenticated"

**Problem**: `gh auth status` fails.

**Solution**:

```bash
gh auth login
# Follow the prompts to authenticate
```bash

```

---

### Issue: "Cannot access GitHub repository"

**Problem**: No remote repository or wrong permissions.

**Solution**:

```bash
# Check remote
git remote -v

# Add remote if missing
git remote add origin git@github.com:USERNAME/REPO.git

# Verify gh can access it
gh repo view
```bash

```

---

### Issue: "You have uncommitted changes"

**Problem**: Working directory is not clean.

**Solution**:

```bash
# See what's changed
git status

# Commit changes
git add .
git commit -m "your message"

# Or stash them
git stash
```bash

```

---

### Issue: "Not on main branch"

**Problem**: Release script expects main/master branch.

**Solution**:

```bash
# Switch to main
git checkout main

# Or modify release.sh to use current branch (already done in latest version)
```bash

```

---

### Issue: "Release already exists"

**Problem**: Tag or release already exists on GitHub.

**Solution**:

```bash
# Delete the release
gh release delete v0.1.0 --yes

# Delete the tag locally
git tag -d v0.1.0

# Delete the tag remotely
git push origin :refs/tags/v0.1.0

# Try again
./release.sh --patch
```bash

```

---

### Issue: "Cross-compilation failed"

**Problem**: Can't build for other architecture.

**Solution**: This is expected and non-fatal. The script will continue with native binary only. GitHub Actions will build other architectures.

To enable local cross-compilation on macOS:

```bash
# If on Intel Mac
rustup target add aarch64-apple-darwin

# If on Apple Silicon
rustup target add x86_64-apple-darwin
```bash

```

---

### Issue: "gh release create failed"

**Problem**: GitHub release creation failed.

**Possible causes**:

1. Network issues
2. Permission issues
3. Tag already exists
4. No binaries found

**Solution**:

```bash
# Check if binaries were built
ls -la target/release-artifacts/

# Check if tag exists
git tag -l

# Try creating release manually
gh release create v0.1.0 --title "v0.1.0" --notes "Release notes" target/release-artifacts/*
```bash

```

---

### Issue: "Binary not found after build"

**Problem**: `target/release/scanner` doesn't exist.

**Solution**:

```bash
# Build manually to see errors
cargo build --release

# Check for compilation errors
cargo check
```bash

```

---

## Verification Steps

Before releasing, run these checks:

```bash
# 1. Check release readiness
./check-release.sh

# 2. Verify build works
cargo build --release

# 3. Test the binary
./target/release/scanner --help

# 4. Verify gh authentication
gh auth status

# 5. Verify repository access
gh repo view

# 6. Do a dry run
./release.sh --patch --dry-run
```bash

```

---

## Manual Release Process

If the script fails, you can release manually:

```bash
# 1. Update version in Cargo.toml
# Edit manually or use sed
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

# 7. Copy binary
cp target/release/scanner target/release-artifacts/scanner-0.1.1-darwin-aarch64

# 8. Create GitHub release
gh release create v0.1.1 \
  --title "v0.1.1" \
  --notes "Release v0.1.1" \
  target/release-artifacts/*
```bash

```

---

## Getting Help

If you're still stuck:

1. Check GitHub Actions logs for CI build failures
2. Run `./check-release.sh` for diagnostics
3. Check `gh` version: `gh --version` (should be 2.0+)
4. Verify Rust toolchain: `rustc --version`
5. Check git configuration: `git config --list`
