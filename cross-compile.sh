#!/usr/bin/env bash
set -euo pipefail

# Cross-compilation script for Scanner
# Builds binaries for: macOS (arm64/amd64), Linux (amd64), Windows (amd64)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Target platforms
TARGETS=(
    "aarch64-apple-darwin"      # macOS ARM64 (Apple Silicon)
    "x86_64-apple-darwin"       # macOS AMD64 (Intel)
    "x86_64-unknown-linux-gnu"  # Linux AMD64
    "i686-unknown-linux-gnu"    # Linux x86 (32-bit)
    "x86_64-pc-windows-gnu"     # Windows AMD64
)

# Display names for targets
declare -A TARGET_NAMES=(
    ["aarch64-apple-darwin"]="macOS ARM64 (Apple Silicon)"
    ["x86_64-apple-darwin"]="macOS AMD64 (Intel)"
    ["x86_64-unknown-linux-gnu"]="Linux AMD64"
    ["i686-unknown-linux-gnu"]="Linux x86 (32-bit)"
    ["x86_64-pc-windows-gnu"]="Windows AMD64"
)

echo -e "${BLUE}=== Scanner Cross-Compilation ===${NC}\n"

# Check if rustup is installed
if ! command -v rustup &> /dev/null; then
    echo -e "${RED}Error: rustup is not installed${NC}"
    echo "Cross-compilation requires rustup. Install it with:"
    echo "  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    echo ""
    echo "See docs/CROSS_COMPILE_SETUP.md for more information."
    exit 1
fi

echo -e "${GREEN}✓${NC} rustup found"

# Setup cargo command - ensure rustup's toolchain is in PATH
if command -v cargo &> /dev/null; then
    CARGO_PATH=$(which cargo)
    RUSTC_SYSROOT=$(rustc --print sysroot 2>&1 || echo "unknown")

    if [[ "$CARGO_PATH" == *"homebrew"* ]] || [[ "$RUSTC_SYSROOT" == *"homebrew"* ]]; then
        echo -e "${RED}Error: Homebrew's Rust is being used instead of rustup${NC}"
        echo ""
        echo "Cross-compilation requires rustup's toolchain."
        echo ""
        echo "Options:"
        echo "  1. Remove Homebrew's Rust (recommended):"
        echo "     brew uninstall rust"
        echo ""
        echo "  2. Adjust your PATH to prioritize rustup:"
        echo "     export PATH=\"\$HOME/.cargo/bin:\$PATH\""
        echo "     Add this to your ~/.zshrc or ~/.bashrc"
        echo ""
        echo "Current cargo: $CARGO_PATH"
        echo "Current sysroot: $RUSTC_SYSROOT"
        echo ""
        exit 1
    fi

    echo -e "${GREEN}✓${NC} Using rustup's toolchain (cargo in PATH)"
else
    # cargo not in PATH, add rustup's toolchain bin to PATH
    TOOLCHAIN_PATH=$(rustup which cargo | xargs dirname)
    export PATH="$TOOLCHAIN_PATH:$PATH"
    echo -e "${GREEN}✓${NC} Using rustup's toolchain (added to PATH)"
fi

# Check and install missing targets
echo -e "\n${BLUE}Checking targets...${NC}"
for target in "${TARGETS[@]}"; do
    if rustup target list --installed | grep -q "^${target}$"; then
        echo -e "${GREEN}✓${NC} ${target} already installed"
    else
        echo -e "${YELLOW}Installing ${target}...${NC}"
        rustup target add "$target"
    fi
done

# Ensure rust-std is available for all targets
echo -e "\n${BLUE}Verifying standard library components...${NC}"
for target in "${TARGETS[@]}"; do
    if rustup component list --installed --toolchain stable | grep -q "rust-std-${target}"; then
        echo -e "${GREEN}✓${NC} rust-std for ${target}"
    else
        echo -e "${YELLOW}Installing rust-std for ${target}...${NC}"
        rustup component add rust-std --target "$target" --toolchain stable 2>&1 || true
    fi
done

# Check if Docker is available
DOCKER_AVAILABLE=false
if command -v docker &> /dev/null; then
    if docker info &> /dev/null; then
        DOCKER_AVAILABLE=true
        echo -e "${GREEN}✓${NC} Docker available for Linux builds"
    fi
fi

# Build for each target
echo -e "\n${BLUE}Building binaries...${NC}\n"
BUILD_ERRORS=()

for target in "${TARGETS[@]}"; do
    echo -e "${BLUE}Building for ${TARGET_NAMES[$target]}...${NC}"

    # Try native build first
    if cargo build --release --target "$target" 2>&1 | tee /tmp/build-${target}.log; then
        echo -e "${GREEN}✓${NC} Built successfully\n"
    else
        echo -e "${RED}✗${NC} Native build failed\n"

        # Try Docker fallback for Linux targets
        if [[ "$target" == *"linux-gnu"* ]] && [ "$DOCKER_AVAILABLE" = true ]; then
            echo -e "  ${BLUE}→${NC} Attempting Docker build...\n"

            # Determine Rust image architecture
            if [[ "$target" == "i686-unknown-linux-gnu" ]]; then
                RUST_IMAGE="rust:latest"
                DOCKER_PLATFORM="--platform linux/386"
            else
                RUST_IMAGE="rust:latest"
                DOCKER_PLATFORM="--platform linux/amd64"
            fi

            # Build using Docker
            if docker run --rm $DOCKER_PLATFORM \
                -v "$SCRIPT_DIR:/workspace" \
                -w /workspace \
                "$RUST_IMAGE" \
                bash -c "rustup target add $target && cargo build --release --target $target" 2>&1 | tee /tmp/build-docker-${target}.log; then
                echo -e "  ${GREEN}✓${NC} Docker build succeeded!\n"
            else
                echo -e "  ${RED}✗${NC} Docker build also failed\n"
                BUILD_ERRORS+=("$target")
            fi
        else
            # Provide helpful error messages
            if [[ "$target" == "x86_64-pc-windows-gnu" ]] && grep -q "dlltool" /tmp/build-${target}.log; then
                echo -e "  ${YELLOW}Hint:${NC} Windows builds require MinGW. Install with:"
                echo -e "  ${YELLOW}brew install mingw-w64${NC}\n"
            elif [[ "$target" == *"linux-gnu"* ]]; then
                if [ "$DOCKER_AVAILABLE" = false ]; then
                    echo -e "  ${YELLOW}Hint:${NC} Linux builds failed. Docker not available."
                    echo -e "  ${YELLOW}Install Docker to enable Linux builds:${NC}"
                    echo -e "  ${YELLOW}https://docs.docker.com/desktop/install/mac-install/${NC}\n"
                else
                    echo -e "  ${YELLOW}Hint:${NC} Linux builds may require a cross-compiler."
                    echo -e "  ${YELLOW}Consider building on Linux directly.${NC}\n"
                fi
            fi

            BUILD_ERRORS+=("$target")
        fi
    fi
done

# Summary
echo -e "\n${BLUE}=== Build Summary ===${NC}\n"

if [ ${#BUILD_ERRORS[@]} -eq 0 ]; then
    echo -e "${GREEN}All builds completed successfully!${NC}\n"
else
    echo -e "${RED}Some builds failed:${NC}"
    for target in "${BUILD_ERRORS[@]}"; do
        echo -e "  ${RED}✗${NC} ${TARGET_NAMES[$target]}"
    done
    echo ""
fi

# List built binaries
echo -e "${BLUE}Built binaries:${NC}\n"
for target in "${TARGETS[@]}"; do
    if [[ ! " ${BUILD_ERRORS[@]} " =~ " ${target} " ]]; then
        binary_path="target/${target}/release/scanner"
        if [ "$target" = "x86_64-pc-windows-gnu" ]; then
            binary_path="${binary_path}.exe"
        fi

        if [ -f "$binary_path" ]; then
            size=$(du -h "$binary_path" | cut -f1)
            echo -e "  ${GREEN}✓${NC} ${TARGET_NAMES[$target]}"
            echo -e "    Path: ${binary_path}"
            echo -e "    Size: ${size}"

            # Show file info (architecture)
            if command -v file &> /dev/null; then
                arch_info=$(file "$binary_path" | cut -d: -f2-)
                echo -e "    Info:${arch_info}"
            fi
            echo ""
        fi
    fi
done

# Create dist directory with all binaries
echo -e "${BLUE}Creating distribution packages...${NC}\n"
DIST_DIR="dist"
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

for target in "${TARGETS[@]}"; do
    if [[ ! " ${BUILD_ERRORS[@]} " =~ " ${target} " ]]; then
        binary_name="scanner"
        if [ "$target" = "x86_64-pc-windows-gnu" ]; then
            binary_name="scanner.exe"
        fi

        binary_path="target/${target}/release/${binary_name}"

        if [ -f "$binary_path" ]; then
            # Create archive name
            case "$target" in
                "aarch64-apple-darwin")
                    archive_name="scanner-v${VERSION}-macos-arm64"
                    ;;
                "x86_64-apple-darwin")
                    archive_name="scanner-v${VERSION}-macos-amd64"
                    ;;
                "x86_64-unknown-linux-gnu")
                    archive_name="scanner-v${VERSION}-linux-amd64"
                    ;;
                "i686-unknown-linux-gnu")
                    archive_name="scanner-v${VERSION}-linux-x86"
                    ;;
                "x86_64-pc-windows-gnu")
                    archive_name="scanner-v${VERSION}-windows-amd64"
                    ;;
            esac

            # Create tar.gz for Unix, zip for Windows
            if [ "$target" = "x86_64-pc-windows-gnu" ]; then
                if command -v zip &> /dev/null; then
                    (cd "target/${target}/release" && zip -q "${SCRIPT_DIR}/${DIST_DIR}/${archive_name}.zip" "$binary_name")
                    echo -e "${GREEN}✓${NC} Created ${archive_name}.zip"
                else
                    echo -e "${YELLOW}⚠${NC}  zip not found, skipping Windows archive"
                fi
            else
                tar -czf "${DIST_DIR}/${archive_name}.tar.gz" -C "target/${target}/release" "$binary_name"
                echo -e "${GREEN}✓${NC} Created ${archive_name}.tar.gz"
            fi
        fi
    fi
done

echo -e "\n${GREEN}Cross-compilation complete!${NC}"
echo -e "Distribution packages are in: ${DIST_DIR}/"

if [ ${#BUILD_ERRORS[@]} -ne 0 ]; then
    exit 1
fi
