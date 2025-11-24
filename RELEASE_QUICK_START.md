# Quick Release Guide

## TL;DR

```bash
# Install GitHub CLI (one-time setup)
brew install gh
gh auth login

# Create a release
./release.sh --patch   # 0.1.0 → 0.1.1
./release.sh --minor   # 0.1.0 → 0.2.0
./release.sh --major   # 0.1.0 → 1.0.0
```

## What Happens

1. Version bumped in `Cargo.toml`
2. Git tag created and pushed
3. Binaries built for your architecture
4. GitHub release created with binaries
5. GitHub Actions builds additional architectures

## View Your Release

```bash
gh release view
```

Or visit: https://github.com/YOUR_USERNAME/scanner/releases

## Common Issues

**"You have uncommitted changes"**
→ Commit or stash your changes first

**"gh: command not found"**
→ Install GitHub CLI: `brew install gh`

**"Not authenticated"**
→ Run: `gh auth login`

See `RELEASE.md` for full documentation.
