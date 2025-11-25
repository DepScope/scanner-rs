#!/usr/bin/env bash
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
REUPLOAD=false

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
        --reupload)
            REUPLOAD=true
            shift
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Usage: ./release-all.sh [--patch|--minor|--major] [--dry-run] [--reupload]"
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

# Check: Uncommitted changes (skip if reupload)
if [ "$REUPLOAD" = false ]; then
    if [ -z "$(git status --porcelain)" ]; then
        echo -e "  ${GREEN}✓${NC} No uncommitted changes"
    else
        echo -e "  ${RED}✗${NC} You have uncommitted changes"
        git status --short | sed 's/^/    /'
        READY=false
    fi
else
    echo -e "  ${YELLOW}⚠${NC} Skipping uncommitted changes check (reupload mode)"
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
    CARGO_PATH=$(which cargo)
    RUSTC_SYSROOT=$(rustc --print sysroot 2>&1 || echo "unknown")

    if [[ "$CARGO_PATH" == *"homebrew"* ]] || [[ "$RUSTC_SYSROOT" == *"homebrew"* ]]; then
        echo -e "  ${YELLOW}⚠${NC} Homebrew's Rust detected (limited to native builds)"
        echo -e "    For cross-compilation, remove Homebrew Rust: ${YELLOW}brew uninstall rust${NC}"
        echo -e "    Or adjust PATH: ${YELLOW}export PATH=\"\$HOME/.cargo/bin:\$PATH\"${NC}"
        CAN_CROSS_COMPILE=false
    else
        ACTIVE_TOOLCHAIN=$(rustup show active-toolchain 2>&1 || true)
        if [[ "$ACTIVE_TOOLCHAIN" == *"no active toolchain"* ]] || [[ "$ACTIVE_TOOLCHAIN" == *"error"* ]]; then
            echo -e "  ${YELLOW}⚠${NC} rustup found but no active toolchain"
            echo -e "    To enable cross-compilation: ${YELLOW}rustup default stable${NC}"
            CAN_CROSS_COMPILE=false
        else
            echo -e "  ${GREEN}✓${NC} rustup configured (cross-compilation enabled)"
            CAN_CROSS_COMPILE=true

            # Ensure cargo is in PATH
            if ! command -v cargo &> /dev/null; then
                TOOLCHAIN_PATH=$(rustup which cargo | xargs dirname)
                export PATH="$TOOLCHAIN_PATH:$PATH"
            fi
        fi
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

# Calculate new version or use current for reupload
if [ "$REUPLOAD" = true ]; then
    NEW_VERSION="$CURRENT_VERSION"
    echo ""
    echo -e "  ${YELLOW}Reupload mode:${NC} Using existing version ${GREEN}${NEW_VERSION}${NC}"
    echo ""
else
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
fi

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
if [ "$REUPLOAD" = false ]; then
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
    git commit -m "chore(release): bump version to ${NEW_VERSION}"
    echo -e "  ${GREEN}✓${NC} Committed version bump"

    git tag -a "v${NEW_VERSION}" -m "Release v${NEW_VERSION}"
    echo -e "  ${GREEN}✓${NC} Created tag v${NEW_VERSION}"

    CURRENT_BRANCH=$(git branch --show-current)
    git push origin "$CURRENT_BRANCH"
    git push origin "v${NEW_VERSION}"
    echo -e "  ${GREEN}✓${NC} Pushed to remote"
else
    echo ""
    echo -e "${GREEN}[2/5] Skipping Version Update (Reupload Mode)${NC}"
    echo ""
    echo -e "  ${YELLOW}⚠${NC} Using existing version ${NEW_VERSION}"
    echo -e "  ${YELLOW}⚠${NC} Checking if tag exists..."

    if git rev-parse "v${NEW_VERSION}" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} Tag v${NEW_VERSION} exists"
    else
        echo -e "  ${RED}✗${NC} Tag v${NEW_VERSION} does not exist"
        echo "Cannot reupload without existing tag. Run without --reupload to create a new release."
        exit 1
    fi
fi

# ============================================================================
# STEP 3: BUILD BINARIES
# ============================================================================
echo ""
echo -e "${GREEN}[3/5] Building Binaries${NC}"
echo ""

RELEASE_DIR="target/release-artifacts"

# Clean old artifacts
if [ -d "$RELEASE_DIR" ]; then
    echo -e "  ${YELLOW}→${NC} Cleaning old artifacts..."
    rm -rf "$RELEASE_DIR"
    echo -e "  ${GREEN}✓${NC} Removed old artifacts"
fi

mkdir -p "$RELEASE_DIR"
echo -e "  ${GREEN}✓${NC} Created clean artifacts directory"
echo ""

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

# Build native binary
echo -e "  ${BLUE}Building native binary...${NC}"
cargo build --release > /dev/null 2>&1
NATIVE_BINARY="scanner-${NEW_VERSION}-${OS}-${ARCH_NAME}"
cp target/release/scanner "$RELEASE_DIR/${NATIVE_BINARY}"
echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${NATIVE_BINARY}${NC} (native)"

# Check if Docker is available for Linux builds
DOCKER_AVAILABLE=false
if command -v docker &> /dev/null; then
    if docker info &> /dev/null 2>&1; then
        DOCKER_AVAILABLE=true
        echo -e "  ${GREEN}✓${NC} Docker available for Linux builds"
    fi
fi

# Cross-compile if possible
if [ "$CAN_CROSS_COMPILE" = true ]; then
    echo ""
    echo -e "  ${BLUE}Cross-compiling for all platforms...${NC}"

    # Define all target platforms
    declare -A TARGETS=(
        ["aarch64-apple-darwin"]="darwin-arm64"
        ["x86_64-apple-darwin"]="darwin-amd64"
        ["x86_64-unknown-linux-gnu"]="linux-amd64"
        ["i686-unknown-linux-gnu"]="linux-x86"
        ["x86_64-pc-windows-gnu"]="windows-amd64"
    )

    for TARGET in "${!TARGETS[@]}"; do
        PLATFORM_NAME="${TARGETS[$TARGET]}"

        # Skip native target if already built
        if [ "$OS" = "darwin" ] && [[ "$TARGET" == *"$ARCH_NAME"* ]]; then
            echo -e "  ${YELLOW}→${NC} Skipping ${PLATFORM_NAME} (already built as native)"
            continue
        fi

        # Install target if needed
        if ! rustup target list --installed | grep -q "$TARGET"; then
            echo -e "  ${YELLOW}→${NC} Installing ${TARGET}..."
            rustup target add "$TARGET" > /dev/null 2>&1
        fi

        # Build
        echo -e "  ${YELLOW}→${NC} Building for ${PLATFORM_NAME}..."

        # Determine binary extension
        BINARY_EXT=""
        if [[ "$TARGET" == *"windows"* ]]; then
            BINARY_EXT=".exe"
        fi

        # Try native build first
        if cargo build --release --target "$TARGET" > /dev/null 2>&1; then
            BINARY_NAME="scanner-${NEW_VERSION}-${PLATFORM_NAME}${BINARY_EXT}"
            cp "target/${TARGET}/release/scanner${BINARY_EXT}" "$RELEASE_DIR/${BINARY_NAME}"
            echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${BINARY_NAME}${NC}"
        else
            # Try Docker fallback for Linux targets
            if [[ "$TARGET" == *"linux-gnu"* ]] && [ "$DOCKER_AVAILABLE" = true ]; then
                echo -e "  ${BLUE}→${NC} Trying Docker build for ${PLATFORM_NAME}..."

                # Determine Docker platform
                if [[ "$TARGET" == "i686-unknown-linux-gnu" ]]; then
                    DOCKER_PLATFORM="--platform linux/386"
                else
                    DOCKER_PLATFORM="--platform linux/amd64"
                fi

                # Build using Docker
                if docker run --rm $DOCKER_PLATFORM \
                    -v "$PWD:/workspace" \
                    -w /workspace \
                    rust:latest \
                    bash -c "rustup target add $TARGET && cargo build --release --target $TARGET" > /dev/null 2>&1; then
                    BINARY_NAME="scanner-${NEW_VERSION}-${PLATFORM_NAME}${BINARY_EXT}"
                    cp "target/${TARGET}/release/scanner${BINARY_EXT}" "$RELEASE_DIR/${BINARY_NAME}"
                    echo -e "  ${GREEN}✓${NC} Built ${YELLOW}${BINARY_NAME}${NC} (Docker)"
                else
                    echo -e "  ${RED}✗${NC} Docker build also failed for ${PLATFORM_NAME}"
                fi
            else
                echo -e "  ${RED}✗${NC} Failed to build for ${PLATFORM_NAME}"
                if [[ "$TARGET" == *"linux-gnu"* ]] && [ "$DOCKER_AVAILABLE" = false ]; then
                    echo -e "      ${YELLOW}Hint:${NC} Install Docker to enable Linux builds"
                fi
            fi
        fi
    done
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

Download the appropriate binary for your system:

### macOS

\`\`\`bash
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-darwin-arm64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/

# Intel
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-darwin-amd64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/
\`\`\`

### Linux

\`\`\`bash
# x86_64 (AMD64)
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-linux-amd64 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/

# x86 (32-bit)
curl -L https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-linux-x86 -o scanner
chmod +x scanner
sudo mv scanner /usr/local/bin/
\`\`\`

### Windows

Download the \`.exe\` file and add it to your PATH, or run directly:

\`\`\`powershell
# Download to current directory
Invoke-WebRequest -Uri https://github.com/${REPO}/releases/download/v${NEW_VERSION}/scanner-${NEW_VERSION}-windows-amd64.exe -OutFile scanner.exe

# Run
.\\scanner.exe --help
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
echo -e "  ${BLUE}Collecting binaries from $RELEASE_DIR...${NC}"
for binary in "$RELEASE_DIR"/*; do
    if [ -f "$binary" ]; then
        BINARY_PATHS+=("$binary")
        SIZE=$(ls -lh "$binary" | awk '{print $5}')
        echo -e "  ${YELLOW}→${NC} Will upload: $(basename "$binary") (${SIZE})"
    fi
done

if [ ${#BINARY_PATHS[@]} -eq 0 ]; then
    echo -e "${RED}✗ No binary files found in $RELEASE_DIR${NC}"
    echo "Contents of $RELEASE_DIR:"
    ls -la "$RELEASE_DIR" || echo "Directory does not exist"
    exit 1
fi

echo -e "  ${GREEN}✓${NC} Found ${#BINARY_PATHS[@]} binary/binaries to upload"

echo ""

if [ "$REUPLOAD" = true ]; then
    echo -e "  ${BLUE}Checking if release exists...${NC}"

    if gh release view "v${NEW_VERSION}" >/dev/null 2>&1; then
        echo -e "  ${YELLOW}→${NC} Release exists, deleting old assets..."

        # Delete existing release to reupload
        gh release delete "v${NEW_VERSION}" --yes >/dev/null 2>&1 || true
        echo -e "  ${GREEN}✓${NC} Deleted old release"
    fi
fi

echo -e "  ${BLUE}Creating GitHub release with ${#BINARY_PATHS[@]} binary/binaries...${NC}"

# Create release with binaries
if echo "$RELEASE_NOTES" | gh release create "v${NEW_VERSION}" \
    --title "v${NEW_VERSION}" \
    --notes-file - \
    "${BINARY_PATHS[@]}" 2>&1; then
    echo -e "  ${GREEN}✓${NC} GitHub release created with ${#BINARY_PATHS[@]} binary/binaries"
else
    echo -e "${RED}✗ Failed to create GitHub release${NC}"
    echo ""
    echo "Troubleshooting:"
    echo "  1. Check if binaries exist:"
    for path in "${BINARY_PATHS[@]}"; do
        if [ -f "$path" ]; then
            echo -e "     ${GREEN}✓${NC} $path"
        else
            echo -e "     ${RED}✗${NC} $path (missing)"
        fi
    done
    echo ""
    echo "  2. Try uploading manually:"
    echo "     gh release create v${NEW_VERSION} ${BINARY_PATHS[*]}"
    echo ""
    echo "  3. Or delete the tag and try again:"
    echo "     git tag -d v${NEW_VERSION}"
    echo "     git push origin :refs/tags/v${NEW_VERSION}"
    exit 1
fi

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
