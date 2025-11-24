# Pre-commit Hooks - Quick Reference

## Setup (One-time)

```bash
./setup-hooks.sh
```bash

```

## Daily Usage

### Before Committing

```bash
# Run all checks manually
make pre-commit

# Or run individual checks
make fmt      # Format code
make clippy   # Lint code
make test     # Run tests
make check    # Check compilation
```bash

```

### Making a Commit

```bash
git add .
git commit -m "feat(module): description"
# Hooks run automatically
```bash

```

### If Hooks Fail

1. **Formatting issues**: Auto-fixed, review and stage changes
2. **Clippy warnings**: Fix manually, then commit again
3. **Test failures**: Fix tests, then commit again
4. **Compilation errors**: Fix errors, then commit again

### Emergency Bypass (Use Sparingly)

```bash
git commit --no-verify -m "WIP: emergency fix"
```bash

```

## Common Commands

| Command | Description |
|---------|-------------|
| `make pre-commit` | Run all checks |
| `make fmt` | Format code |
| `make clippy` | Run linter |
| `make test` | Run tests |
| `make check` | Check compilation |
| `cargo fmt --all` | Auto-fix formatting |
| `cargo clippy --fix` | Auto-fix some clippy issues |

## Commit Message Format

```bash
<type>(<scope>): <description>

[optional body]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Examples**:

- `feat(parser): add Python metadata parser`
- `fix(indexer): handle nested node_modules`
- `docs(readme): update installation instructions`
- `test(classifier): add version mismatch tests`

## Troubleshooting

### Hook doesn't run

```bash
chmod +x .git/hooks/pre-commit
```bash

```

### Hook takes too long

- Normal: 10-30 seconds
- Use `git commit --no-verify` for WIP commits

### Clippy false positives

```rust
#[allow(clippy::lint_name)]
fn my_function() {}
```bash

```

### Need to update hooks

```bash
git pull
./setup-hooks.sh
```bash

```

## Optional Tools

Install for enhanced functionality:

```bash
# Code statistics
cargo install tokei

# Test coverage
cargo install cargo-tarpaulin

# Advanced hooks
pip install pre-commit
```bash

```

## Help

Full documentation: [PRE_COMMIT_HOOKS.md](PRE_COMMIT_HOOKS.md)
