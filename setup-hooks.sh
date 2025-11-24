#!/bin/bash
# Setup script for pre-commit hooks

set -e

echo "Setting up pre-commit hooks for Rust project..."
echo ""

# Check if git repository
if [ ! -d .git ]; then
    echo "Error: Not a git repository"
    exit 1
fi

# Install pre-commit hook
echo "[1/4] Installing pre-commit hook..."
cp .git/hooks/pre-commit .git/hooks/pre-commit.backup 2>/dev/null || true
chmod +x .git/hooks/pre-commit
echo "  ✓ Pre-commit hook installed"

# Check for optional tools
echo ""
echo "[2/4] Checking for optional tools..."

MISSING_TOOLS=()

if ! command -v tokei &> /dev/null; then
    echo "  ⚠ tokei not found (optional - for detailed code statistics)"
    echo "    Install: cargo install tokei"
    MISSING_TOOLS+=("tokei")
else
    echo "  ✓ tokei found"
fi

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "  ⚠ cargo-tarpaulin not found (optional - for test coverage)"
    echo "    Install: cargo install cargo-tarpaulin"
    MISSING_TOOLS+=("cargo-tarpaulin")
else
    echo "  ✓ cargo-tarpaulin found"
fi

if ! command -v pre-commit &> /dev/null; then
    echo "  ⚠ pre-commit not found (optional - for advanced hooks)"
    echo "    Install: pip install pre-commit"
    MISSING_TOOLS+=("pre-commit")
else
    echo "  ✓ pre-commit found"
fi

# Install pre-commit framework hooks if available
echo ""
echo "[3/4] Setting up pre-commit framework..."
if command -v pre-commit &> /dev/null; then
    if [ -f .pre-commit-config.yaml ]; then
        pre-commit install
        pre-commit install --hook-type commit-msg
        echo "  ✓ Pre-commit framework configured"
    fi
else
    echo "  ⚠ Skipping (pre-commit not installed)"
fi

# Test the hook
echo ""
echo "[4/4] Testing pre-commit hook..."
if .git/hooks/pre-commit; then
    echo "  ✓ Pre-commit hook test passed"
else
    echo "  ✗ Pre-commit hook test failed"
    echo "    Fix the issues and run this script again"
    exit 1
fi

# Summary
echo ""
echo "╔════════════════════════════════════════╗"
echo "║     Pre-commit Setup Complete! ✓       ║"
echo "╚════════════════════════════════════════╝"
echo ""
echo "The following checks will run on every commit:"
echo "  1. Code formatting (cargo fmt)"
echo "  2. Linting (cargo clippy)"
echo "  3. Compilation (cargo check)"
echo "  4. Tests (cargo test)"
echo "  5. Code analytics (LOC, TODO, unwrap)"
echo ""

if [ ${#MISSING_TOOLS[@]} -gt 0 ]; then
    echo "Optional tools not installed:"
    for tool in "${MISSING_TOOLS[@]}"; do
        echo "  - $tool"
    done
    echo ""
    echo "Install them for enhanced functionality."
    echo ""
fi

echo "To skip pre-commit checks (not recommended):"
echo "  git commit --no-verify"
echo ""
echo "To run checks manually:"
echo "  .git/hooks/pre-commit"
echo ""
