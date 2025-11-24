#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default values
VERSION_BUMP="patch"
DRY_RUN=false

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --patch)
            VERSION_BUMP="patch"
            shift
            ;;
        --minor)
            VERSION_BUMP="minor"
            shift
            ;;
        --major)
            VERSION_BUMP="major"
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: ./release-all.sh [--patch|--minor|--major] [--dry-run]"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   Scanner Multi-Platform Release      ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""

# ============================================================================
# STEP 1: PRE-FLIGHT CHECKS
# ============================================================================
echo -e "${GREEN}[1/5] Pre-flight Checks${NC}"
echo ""

READY=true

# Check: Git repository
if [ -d .git ]; then
    echo -e "  ${GREEN}✓${NC} Git repository found"
else
    echo -e "  ${RED}✗${NC} Not a git repository"
    READY=false
fi

# Check: Uncommitted changes
if [ -z "$(git status --porcelain)" ]; then
    echo -e "  ${GREEN}✓${NC} No uncommitted changes"
else
    echo -e "  ${RED}✗${NC} You have uncommitted changes"
    git status --short | sed 's/^/    /'
    READY=false
fi

# Check: GitHub CLI
if command -v gh &> /dev/null; then
    echo -e "  ${GREEN}✓${NC} GitHub CLI installed"

    if gh auth status &> /dev/null; then
        echo -e "  ${GREEN}✓${NC} GitHub CLI authenticated"
    else
        echo -e "  ${RED}✗${NC} GitHub CLI not authenticated (run: gh auth login)"
        READY=false
    fi

    if gh repo view &> /dev/null; then
        REPO=$(gh repo view --json nameWithOwner -q .nameWithOwner)
        echo -e "  ${GREEN}✓${NC} Repository: ${YELLOW}${REPO}${NC}"
    else
        echo -e "  ${RED}✗${NC} Cannot access GitHub repository"
        READY=false
    fi
else
    echo -e "  ${RED}✗${NC} GitHub CLI not installed"
    READY=false
fi

# Check: Rust toolchain
if command -v cargo &> /dev/null; then
    echo -e "  ${GREEN}✓${NC} Rust toolchain installed"
else
    echo -e "  ${RED}✗${NC} Rust toolchain not installed"
    READY=false
fi

# Check: Current version
if [ -f Cargo.toml ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo -e "  ${GREEN}✓${NC} Current version: ${YELLOW}${CURRENT_VERSION}${NC}"
else
    echo -e "  ${RED}✗${NC} Cargo.toml not found"
    READY=false
fi

# Check: Cross-compilation capability
if command -v rustup &> /dev/null; then
    ACTIVE_TOOLCHAIN=$(rustup show active-toolchain 2>&1 || true)
    if [[ "$ACTIVE_TOOLCHAIN" == *"no active toolchain"* ]] || [[ "$ACTIVE_TOOLCHAIN" == *"error"* ]]; then
        echo -e "  ${YELLOW}⚠${NC} rustup found but no active toolchain"
        echo -e "    Using Homebrew Rust - limited to native builds"
        echo -e "    To enable cross-compilation: ${YELLOW}rustup default stable${NC}"
        CAN_CROSS_COMPILE=false
    else
        echo -e "  ${GREEN}✓${NC} rustup configured (cross-compilation enabled)"
        CAN_CROSS_COMPILE=true
    fi
else
    echo -e "  ${YELLOW}⚠${NC} rustup not found (limited to native builds)"
    CAN_CROSS_COMPILE=false
fi

if [ "$READY" = false ]; then
    echo ""
    echo -e "${RED}✗ Pre-flight checks failed${NC}"
    exit 1
fi

# Calculate new version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
case $VERSION_BUMP in
    major)
        MAJOR=$((MAJOR + 1))
        MINOR=0
        PATCH=0
        ;;
    minor)
        MINOR=$((MINOR + 1))
        PATCH=0
        ;;
    patch)
        PATCH=$((PATCH + 1))
        ;;
esac
NEW_VERSION="${MAJOR}.${MINOR}.${PATCH}"

echo ""
echo -e "  ${BLUE}Version bump:${NC} ${YELLOW}${CURRENT_VERSION}${NC} → ${GREEN}${NEW_VERSION}${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN - No changes will be made${NC}"
    exit 0
fi

read -p "Continue with release? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Release cancelled"
    exit 0
fi

# ============================================================================
# STEP 2: UPDATE VERSION
# ============================================================================
echo ""
echo -e "${GREEN}[2/5] Updating Version${NC}"
echo ""

sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
rm Cargo.toml.bak
echo -e "  ${GREEN}✓${NC} Updated Cargo.toml"

cargo build --release > /dev/null 2>&1
echo -e "  ${GREEN}✓${NC} Updated Cargo.lock"

git add Cargo.toml
if [ -f Cargo.lock ]; then
    git add -f Cargo.lock
fi
git commit -m "chore: bump version to ${NEW_VERSION}"
echo -e "  ${GREEN}✓${NC} Committed version bump"

git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"
echo -e "  ${GREEN}✓${NC} Created tag v${NEW_VERSION}"

CURRENT_BRANCH=$(git branch --show-current)
git push origin "$CURRENT_BRANCH"
git push origin "v${NEW_VERSION}"
echo -e "  ${GREEN}✓${NC} Pushed to remote"

# ============================================================================
# STEP 3: BUILD BINARIES
# ============================================================================
echo ""
echo -e "${GREEN}[3/5] Building Binaries${NC}"
echo ""

RELEASE_DIR="target/release-artifacts"
mkdir -p "$RELEASE_DIR"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

# Map architecture names
case $ARCH in
    x86_64)
        ARCH_NAME="x86_64"
        ;;
    arm64|aarch64)
        ARCH_NAME="aarch64"
        ;;
    *)
        ARCH_NAME=$ARCH
        ;;
esac

# Build native binary (already built during version update)
NATIVE_BINARY="scanner-${NEW_VERSION}-${OS}-${ARCH_NAME}"
cp target/release/scanner "$RELEASE_DIR/${NATIVE_BINARY}"
echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${NATIVE_BINARY}${NC} (native)"

# Cross-compile if possible
if [ "$CAN_CROSS_COMPILE" = true ] && [ "$OS" = "darwin" ]; then
    echo ""
    echo -e "  ${BLUE}Cross-compiling for macOS...${NC}"

    TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")

    for TARGET in "${TARGETS[@]}"; do
        # Skip native target
        if [[ "$TARGET" == *"$ARCH_NAME"* ]]; then
            continue
        fi

        TARGET_ARCH=$(echo "$TARGET" | cut -d'-' -f1)

        # Install target if needed
        if ! rustup target list --installed | grep -q "$TARGET"; then
            echo -e "  ${YELLOW}→${NC} Installing ${TARGET}..."
            rustup target add "$TARGET" > /dev/null 2>&1
        fi

        # Build
        echo -e "  ${YELLOW}→${NC} Building for ${TARGET}..."
        if cargo build --release --target "$TARGET" > /dev/null 2>&1; then
            BINARY_NAME="scanner-${NEW_VERSION}-darwin-${TARGET_ARCH}"
            cp "target/${TARGET}/release/scanner" "$RELEASE_DIR/${BINARY_NAME}"
            echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${BINARY_NAME}${NC}"
        else
            echo -e "  ${RED}✗${NC} Failed to build for ${TARGET}"
        fi
    done
elif [ "$CAN_CROSS_COMPILE" = true ] && [ "$OS" = "linux" ]; then
    if command -v cross &> /dev/null; then
        echo ""
        echo -e "  ${BLUE}Cross-compiling for Linux...${NC}"

        TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")

        for TARGET in "${TARGETS[@]}"; do
            if [[ "$TARGET" == *"$ARCH_NAME"* ]]; then
                continue
            fi

            TARGET_ARCH=$(echo "$TARGET" | cut -d'-' -f1)

            echo -e "  ${YELLOW}→${NC} Building for ${TARGET}..."
            if cross build --release --target "$TARGET" > /dev/null 2>&1; then
                BINARY_NAME="scanner-${NEW_VERSION}-linux-${TARGET_ARCH}"
                cp "target/${TARGET}/release/scanner" "$RELEASE_DIR/${BINARY_NAME}"
                echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${BINARY_NAME}${NC}"
            else
                echo -e "  ${RED}✗${NC} Failed to build for ${TARGET}"
            fi
        done
    fi
else
    echo ""
    echo -e "  ${YELLOW}⚠${NC} Cross-compilation not available"
    echo -e "    Install rustup to enable: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

# ============================================================================
# STEP 4: SUMMARY
# ============================================================================
echo ""
echo -e "${GREEN}[4/5] Build Summary${NC}"
echo ""

BINARY_COUNT=$(ls -1 "$RELEASE_DIR" | wc -l | tr -d ' ')
echo -e "  ${GREEN}✓${NC} Created ${YELLOW}${BINARY_COUNT}${NC} binary/binaries:"
echo ""

for binary in "$RELEASE_DIR"/*; do
    SIZE=$(ls -lh "$binary" | awk '{print $5}')
    NAME=$(basename "$binary")
    ARCH_INFO=$(file "$binary" | grep -o 'arm64\|x86_64' || echo "unknown")
    echo -e "    ${YELLOW}${NAME}${NC}"
    echo -e "      Size: ${SIZE}, Arch: ${ARCH_INFO}"
done

# ============================================================================
# STEP 5: CREATE GITHUB RELEASE
# ============================================================================
echo ""
echo -e "${GREEN}[5/5] Creating GitHub Release${NC}"
echo ""

# Create release notes
RELEASE_NOTES="Release v${NEW_VERSION}

## Installation

Download the appropriate binary for your system and make it executable:

\`\`\`bash
# macOS Apple Silicon (M1/M2/M3)
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-darwin-aarch64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/

# macOS Intel
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-darwin-x86_64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/

# Linux x86_64
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-linux-x86_64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/
\`\`\`

## Binaries

This release includes ${BINARY_COUNT} pre-built binary/binaries for:
$(for binary in "$RELEASE_DIR"/*; do echo "- $(basename "$binary")"; done)

## Changes

See commit history for details.
"

if [ -z "$(ls -A "$RELEASE_DIR")" ]; then
    echo -e "${RED}✗ No binaries found${NC}"
    exit 1
fi

# Collect all binary paths
BINARY_PATHS=()
for binary in "$RELEASE_DIR"/*; do
    if [ -f "$binary" ]; then
        BINARY_PATHS+=("$binary")
        echo -e "  ${YELLOW}→${NC} Will upload: $(basename "$binary")"
    fi
done

if [ ${#BINARY_PATHS[@]} -eq 0 ]; then
    echo -e "${RED}✗ No binary files found in $RELEASE_DIR${NC}"
    exit 1
fi

echo ""
echo -e "  ${BLUE}Creating GitHub release with ${#BINARY_PATHS[@]} binary/binaries...${NC}"

echo "$RELEASE_NOTES" | gh release create "v${NEW_VERSION}" \
    --title "v${NEW_VERSION}" \
    --notes-file - \
    "${BINARY_PATHS[@]}" || {
        echo -e "${RED}✗ Failed to create GitHub release${NC}"
        echo "You may need to delete the tag and try again:"
        echo "  git tag -d v${NEW_VERSION}"
        echo "  git push origin :refs/tags/v${NEW_VERSION}"
        exit 1
    }

echo -e "  ${GREEN}✓${NC} GitHub release created with ${#BINARY_PATHS[@]} binary/binaries"

# ============================================================================
# DONE
# ============================================================================
echo ""
echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║         Release Complete! 🎉           ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo ""
echo -e "  Version: ${GREEN}v${NEW_VERSION}${NC}"
echo -e "  Binaries: ${YELLOW}${BINARY_COUNT}${NC}"
echo -e "  Release: ${YELLOW}https://github.com/${REPO}/releases/tag/v${NEW_VERSION}${NC}"
echo ""
