# Release Documentation Index

Quick reference for all release-related documentation.

## Main Documentation

📘 **[RELEASING.md](../RELEASING.md)** - Start here!

- Complete release guide
- Quick start commands
- Prerequisites
- What the script does
- Troubleshooting

## Specialized Guides

🔧 **[CROSS_COMPILE_SETUP.md](../CROSS_COMPILE_SETUP.md)**

- Setting up cross-compilation
- rustup vs Homebrew Rust
- Adding compilation targets
- Platform-specific instructions

🐛 **[TROUBLESHOOTING.md](../TROUBLESHOOTING.md)**

- Common issues and solutions
- Error messages explained
- Manual release process
- Recovery procedures

## Quick Reference

### Create a Release

```bash
./release-all.sh --patch   # Bug fixes
./release-all.sh --minor   # New features
./release-all.sh --major   # Breaking changes
```

### Enable Cross-Compilation

```bash
rustup default stable
rustup target add x86_64-apple-darwin aarch64-apple-darwin
```

### Test Before Release

```bash
./release-all.sh --patch --dry-run
```

## Files

| File | Purpose |
|------|---------|
| `release-all.sh` | Main release script |
| `RELEASING.md` | Complete release guide |
| `CROSS_COMPILE_SETUP.md` | Cross-compilation setup |
| `TROUBLESHOOTING.md` | Problem solving |
| `Makefile` | Convenient shortcuts |
| `.github/RELEASE_CHECKLIST.md` | Release checklist |
| `.github/workflows/release.yml` | CI/CD workflow |

## Workflow

1. **First time setup:**
   - Read [RELEASING.md](../RELEASING.md)
   - Optionally setup cross-compilation: [CROSS_COMPILE_SETUP.md](../CROSS_COMPILE_SETUP.md)

2. **Creating releases:**
   - Use `./release-all.sh --patch` (or --minor/--major)
   - Or use `make release-patch` shortcuts

3. **If issues occur:**
   - Check [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)
   - Check [RELEASING.md](../RELEASING.md) troubleshooting section

## Support

- All documentation is in the repository root
- Check the specific guide for your issue
- Use `--dry-run` flag to test without making changes
