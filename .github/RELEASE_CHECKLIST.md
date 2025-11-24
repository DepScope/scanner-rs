# Release Checklist

Use this checklist when creating a new release.

## Pre-Release

- [ ] All changes committed and pushed
- [ ] Tests passing: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] No clippy warnings: `cargo clippy`
- [ ] Documentation updated (README, CHANGELOG, etc.)
- [ ] On main/master branch (or intended release branch)

## Release Readiness Check

```bash
./check-release.sh
```

Expected output: All green checkmarks ✓

## Create Release

Choose version bump type:

```bash
# Patch release (bug fixes): 0.1.0 → 0.1.1
./release.sh --patch

# Minor release (new features): 0.1.0 → 0.2.0
./release.sh --minor

# Major release (breaking changes): 0.1.0 → 1.0.0
./release.sh --major
```

Or use make:

```bash
make release-patch
make release-minor
make release-major
```

## Post-Release

- [ ] Verify release on GitHub: `gh release view`
- [ ] Check GitHub Actions completed successfully
- [ ] Test download and installation of binaries
- [ ] Update any dependent projects
- [ ] Announce release (if applicable)

## If Something Goes Wrong

See [TROUBLESHOOTING.md](../../TROUBLESHOOTING.md) for common issues and solutions.

Quick rollback:
```bash
# Delete release
gh release delete vX.Y.Z --yes

# Delete tag
git tag -d vX.Y.Z
git push origin :refs/tags/vX.Y.Z

# Revert version commit
git revert HEAD
git push
```
