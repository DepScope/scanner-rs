# Pre-commit Hooks for Rust

This project uses pre-commit hooks to ensure code quality and consistency before commits are made.

## Quick Setup

```bash
# Run the setup script
./setup-hooks.sh
```bash

```

That's it! The hooks will now run automatically on every commit.

## What Gets Checked

Every commit triggers the following checks:

### 1. Code Formatting (cargo fmt)

- Ensures all Rust code follows standard formatting
- **Auto-fix**: Yes - automatically formats code if issues found
- **Bypass**: Not recommended

### 2. Linting (cargo clippy)

- Checks for common mistakes and non-idiomatic code
- Treats warnings as errors
- **Auto-fix**: No - manual fixes required
- **Common issues**: Unused variables, inefficient code patterns

### 3. Compilation (cargo check)

- Verifies code compiles successfully
- Checks all features and targets
- **Auto-fix**: No - fix compilation errors manually

### 4. Testing (cargo test)

- Runs all unit and integration tests
- Must pass 100% to commit
- **Auto-fix**: No - fix failing tests manually

### 5. Code Analytics

- **Lines of Code**: Total LOC count (requires `tokei` for details)
- **TODO/FIXME**: Warns if TODO comments found
- **unwrap() Usage**: Warns about potential panics
- **Test Coverage**: Shows coverage % (requires `cargo-tarpaulin`)

## Manual Usage

Run checks manually without committing:

```bash
# Run all checks
.git/hooks/pre-commit

# Run specific checks
cargo fmt --all -- --check    # Check formatting
cargo clippy --all-features   # Run linter
cargo check                   # Check compilation
cargo test                    # Run tests
```bash

```

## Skipping Hooks

**Not recommended**, but you can skip hooks with:

```bash
git commit --no-verify
```bash

```

Only use this for:

- Work-in-progress commits on feature branches
- Emergency hotfixes (fix the issues in next commit)
- Commits that intentionally break tests temporarily

## Optional Tools

Install these for enhanced functionality:

### tokei (Code Statistics)

```bash
cargo install tokei
```bash

```

Provides detailed code statistics:

- Lines of code by language
- Comments vs code ratio
- Blank lines

### cargo-tarpaulin (Test Coverage)

```bash
cargo install cargo-tarpaulin
```bash

```

Measures test coverage:

- Line coverage percentage
- Identifies untested code
- Generates coverage reports

### pre-commit Framework (Advanced)

```bash
pip install pre-commit
```bash

```

Enables additional hooks:

- YAML/TOML/JSON validation
- Markdown linting
- Commit message format checking
- Large file detection
- Private key detection

After installing, run:

```bash
pre-commit install
pre-commit install --hook-type commit-msg
```bash

```

## Configuration Files

### `.git/hooks/pre-commit`

The main pre-commit hook script. Runs all checks in sequence.

### `.pre-commit-config.yaml`

Configuration for the pre-commit framework (optional).

Includes:

- Rust formatting and linting
- File format validation
- Markdown linting
- Conventional commit message checking

### `setup-hooks.sh`

Setup script that installs and configures all hooks.

## Troubleshooting

### Hook doesn't run

```bash
# Check if hook is executable
ls -la .git/hooks/pre-commit

# Make it executable
chmod +x .git/hooks/pre-commit
```bash

```

### Formatting issues

```bash
# Auto-fix formatting
cargo fmt --all

# Check what would change
cargo fmt --all -- --check
```bash

```

### Clippy warnings

```bash
# See all warnings
cargo clippy --all-features --all-targets

# Fix automatically (some issues)
cargo clippy --fix --all-features --all-targets
```bash

```

### Tests failing

```bash
# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific module
cargo test module_name
```bash

```

### Hook takes too long

The hook runs all checks which can take 10-30 seconds. To speed up:

1. **Use incremental compilation** (enabled by default)
2. **Run checks in parallel** (already done)
3. **Skip tests temporarily** (edit `.git/hooks/pre-commit` to comment out test section)

### False positives

If clippy reports false positives:

```rust
// Allow specific lint for one item
#[allow(clippy::lint_name)]
fn my_function() {}

// Allow for entire file
#![allow(clippy::lint_name)]
```bash

```

## Commit Message Format

If using the pre-commit framework, commit messages should follow [Conventional Commits](https://www.conventionalcommits.org/):

```bash
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Test changes
- `chore`: Build/tooling changes

**Examples:**

```bash
git commit -m "feat(parser): add Python metadata parser"
git commit -m "fix(indexer): handle nested node_modules"
git commit -m "docs(readme): update installation instructions"
git commit -m "test(classifier): add version mismatch tests"
```bash

```

## CI/CD Integration

These same checks should run in CI/CD:

```yaml
# Example GitHub Actions workflow
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo fmt --all -- --check
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo test
```bash

```

## Best Practices

1. **Run checks before committing**: Even with hooks, run `cargo test` during development
2. **Fix issues immediately**: Don't accumulate warnings
3. **Write tests first**: TDD helps catch issues early
4. **Keep commits small**: Easier to debug if hooks fail
5. **Use meaningful commit messages**: Helps with debugging and history

## Disabling Hooks

To temporarily disable hooks:

```bash
# Rename the hook
mv .git/hooks/pre-commit .git/hooks/pre-commit.disabled

# Re-enable
mv .git/hooks/pre-commit.disabled .git/hooks/pre-commit
```bash

```

To permanently remove:

```bash
rm .git/hooks/pre-commit
```bash

```

## Updating Hooks

To update the hooks:

```bash
# Pull latest changes
git pull

# Re-run setup
./setup-hooks.sh
```bash

```

## Support

If you encounter issues:

1. Check this documentation
2. Run checks manually to see detailed errors
3. Check the [Rust documentation](https://doc.rust-lang.org/)
4. Ask the team for help

## Resources

- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Clippy Lints](https://rust-lang.github.io/rust-clippy/master/)
- [rustfmt Configuration](https://rust-lang.github.io/rustfmt/)
- [Pre-commit Framework](https://pre-commit.com/)
- [Conventional Commits](https://www.conventionalcommits.org/)
