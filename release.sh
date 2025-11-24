#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default values
VERSION_BUMP="patch"
DRY_RUN=false
SKIP_CROSS_COMPILE=false

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
        --no-cross-compile)
            SKIP_CROSS_COMPILE=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: ./release.sh [--patch|--minor|--major] [--dry-run] [--no-cross-compile]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}=== Scanner Release Script ===${NC}"
echo -e "Version bump: ${YELLOW}${VERSION_BUMP}${NC}"
echo ""

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo -e "${RED}Error: Not in a git repository${NC}"
    exit 1
fi

# Check for uncommitted changes
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${RED}Error: You have uncommitted changes${NC}"
    git status --short
    exit 1
fi

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI (gh) is not installed${NC}"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if gh is authenticated
if ! gh auth status &> /dev/null; then
    echo -e "${RED}Error: GitHub CLI is not authenticated${NC}"
    echo "Run: gh auth login"
    exit 1
fi

# Verify we can access the repository
if ! gh repo view &> /dev/null; then
    echo -e "${RED}Error: Cannot access GitHub repository${NC}"
    echo "Make sure you have a remote repository configured"
    exit 1
fi

# Get current version from Cargo.toml
CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
echo -e "Current version: ${YELLOW}${CURRENT_VERSION}${NC}"

# Parse version components
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"

# Bump version
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
echo -e "New version: ${GREEN}${NEW_VERSION}${NC}"
echo ""

if [ "$DRY_RUN" = true ]; then
    echo -e "${YELLOW}DRY RUN - No changes will be made${NC}"
    exit 0
fi

# Update version in Cargo.toml
echo -e "${GREEN}Updating Cargo.toml...${NC}"
sed -i.bak "s/^version = \".*\"/version = \"${NEW_VERSION}\"/" Cargo.toml
rm Cargo.toml.bak

# Update Cargo.lock
echo -e "${GREEN}Updating Cargo.lock...${NC}"
cargo build --release

# Commit version bump
echo -e "${GREEN}Committing version bump...${NC}"
git add Cargo.toml
# Add Cargo.lock if it exists and is tracked
if [ -f Cargo.lock ] && git ls-files --error-unmatch Cargo.lock &>/dev/null; then
    git add Cargo.lock
elif [ -f Cargo.lock ]; then
    # Force add if it exists but is ignored
    git add -f Cargo.lock
fi
git commit -m "chore: bump version to ${NEW_VERSION}"

# Create git tag
echo -e "${GREEN}Creating git tag v${NEW_VERSION}...${NC}"
git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"

# Push changes and tags
echo -e "${GREEN}Pushing to remote...${NC}"
CURRENT_BRANCH=$(git branch --show-current)
git push origin "$CURRENT_BRANCH"
git push origin "v${NEW_VERSION}"

echo ""
echo -e "${GREEN}=== Building Release Binaries ===${NC}"

# Create release directory
RELEASE_DIR="target/release-artifacts"
mkdir -p "$RELEASE_DIR"

# Build for current architecture (native)
echo -e "${GREEN}Building for native architecture...${NC}"
cargo build --release

# Detect current architecture
ARCH=$(uname -m)
OS=$(uname -s | tr '[:upper:]' '[:lower:]')

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

# Copy native binary
NATIVE_BINARY="scanner-${NEW_VERSION}-${OS}-${ARCH_NAME}"
cp target/release/scanner "$RELEASE_DIR/${NATIVE_BINARY}"
echo -e "Created: ${YELLOW}${NATIVE_BINARY}${NC}"

# Cross-compile for other architectures
if [ "$SKIP_CROSS_COMPILE" = true ]; then
    echo ""
    echo -e "${YELLOW}Skipping cross-compilation (--no-cross-compile flag)${NC}"
else
    echo ""
    echo -e "${GREEN}=== Cross-Compilation ===${NC}"

    if [ "$OS" = "darwin" ]; then
    # macOS: Build for both Intel and Apple Silicon
    
    # Check if rustup is available
    if command -v rustup &> /dev/null; then
        TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")
        
        for TARGET in "${TARGETS[@]}"; do
            # Skip if this is the native target (already built)
            if [[ "$TARGET" == *"$ARCH_NAME"* ]]; then
                continue
            fi
            
            echo -e "${GREEN}Building for ${TARGET}...${NC}"
            
            # Install target if needed
            if ! rustup target list --installed | grep -q "$TARGET"; then
                echo -e "Installing target ${TARGET}..."
                rustup target add "$TARGET"
            fi
            
            # Build
            if cargo build --release --target "$TARGET"; then
                TARGET_ARCH=$(echo "$TARGET" | cut -d'-' -f1)
                BINARY_NAME="scanner-${NEW_VERSION}-darwin-${TARGET_ARCH}"
                cp "target/${TARGET}/release/scanner" "$RELEASE_DIR/${BINARY_NAME}"
                echo -e "${GREEN}✓${NC} Created: ${YELLOW}${BINARY_NAME}${NC}"
            else
                echo -e "${RED}✗${NC} Failed to build for ${TARGET}"
            fi
        done
    else
        echo -e "${YELLOW}rustup not found - cross-compilation not available${NC}"
        echo -e "${YELLOW}You have Rust installed via Homebrew${NC}"
        echo -e "To enable cross-compilation, install rustup:"
        echo -e "  ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        echo -e "Then add targets:"
        echo -e "  ${YELLOW}rustup target add x86_64-apple-darwin aarch64-apple-darwin${NC}"
    fi
    
elif [ "$OS" = "linux" ]; then
    # Linux: Can cross-compile with cross tool
    echo -e "${YELLOW}Note: Linux cross-compilation requires 'cross' tool${NC}"
    echo -e "Install with: cargo install cross"
    echo ""
    
    if command -v cross &> /dev/null; then
        TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
        
        for TARGET in "${TARGETS[@]}"; do
            # Skip if this is the native target
            if [[ "$TARGET" == *"$ARCH_NAME"* ]]; then
                continue
            fi
            
            echo -e "${GREEN}Building for ${TARGET}...${NC}"
            
            if cross build --release --target "$TARGET"; then
                TARGET_ARCH=$(echo "$TARGET" | cut -d'-' -f1)
                BINARY_NAME="scanner-${NEW_VERSION}-linux-${TARGET_ARCH}"
                cp "target/${TARGET}/release/scanner" "$RELEASE_DIR/${BINARY_NAME}"
                echo -e "${GREEN}✓${NC} Created: ${YELLOW}${BINARY_NAME}${NC}"
            else
                echo -e "${RED}✗${NC} Failed to build for ${TARGET}"
            fi
        done
    else
        echo -e "${YELLOW}Skipping Linux cross-compilation (cross tool not installed)${NC}"
    fi
    fi
fi

echo ""
echo -e "${GREEN}=== Creating GitHub Release ===${NC}"

# Create release notes
RELEASE_NOTES="Release v${NEW_VERSION}

## Installation

Download the appropriate binary for your system:
- macOS Apple Silicon (M1/M2): \`scanner-${NEW_VERSION}-darwin-aarch64\`
- macOS Intel: \`scanner-${NEW_VERSION}-darwin-x86_64\`
- Linux x86_64: \`scanner-${NEW_VERSION}-linux-x86_64\`

Make the binary executable:
\`\`\`bash
chmod +x scanner-*
\`\`\`

Move to your PATH:
\`\`\`bash
sudo mv scanner-* /usr/local/bin/scanner
\`\`\`

## Changes

See commit history for details.
"

# Create GitHub release with all binaries
if [ -z "$(ls -A "$RELEASE_DIR")" ]; then
    echo -e "${RED}Error: No binaries found in $RELEASE_DIR${NC}"
    exit 1
fi

echo "$RELEASE_NOTES" | gh release create "v${NEW_VERSION}" \
    --title "v${NEW_VERSION}" \
    --notes-file - \
    "$RELEASE_DIR"/* || {
        echo -e "${RED}Error: Failed to create GitHub release${NC}"
        echo "You may need to delete the tag and try again:"
        echo "  git tag -d v${NEW_VERSION}"
        echo "  git push origin :refs/tags/v${NEW_VERSION}"
        exit 1
    }

echo ""
echo -e "${GREEN}=== Release Complete! ===${NC}"
echo -e "Version: ${YELLOW}v${NEW_VERSION}${NC}"
echo -e "View release: ${YELLOW}https://github.com/$(gh repo view --json nameWithOwner -q .nameWithOwner)/releases/tag/v${NEW_VERSION}${NC}"
echo ""
echo -e "Binaries created:"
ls -lh "$RELEASE_DIR"
