#!/usr/bin/env bash
set -euo pipefail

# Docker-based Linux build script for Scanner
# Builds Linux binaries using Docker containers

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Scanner Docker Linux Builds ===${NC}\n"

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo -e "${RED}Error: Docker is not installed${NC}"
    echo "Install Docker Desktop from: https://docs.docker.com/desktop/install/mac-install/"
    exit 1
fi

if ! docker info &> /dev/null; then
    echo -e "${RED}Error: Docker daemon is not running${NC}"
    echo "Start Docker Desktop and try again"
    exit 1
fi

echo -e "${GREEN}✓${NC} Docker is available\n"

# Linux targets to build
declare -A LINUX_TARGETS=(
    ["x86_64-unknown-linux-gnu"]="linux-amd64"
    ["i686-unknown-linux-gnu"]="linux-x86"
)

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')

echo -e "${BLUE}Building Linux binaries with Docker...${NC}\n"

BUILD_ERRORS=()

for target in "${!LINUX_TARGETS[@]}"; do
    platform_name="${LINUX_TARGETS[$target]}"

    echo -e "${BLUE}Building for ${platform_name}...${NC}"

    # Determine Docker platform
    if [[ "$target" == "i686-unknown-linux-gnu" ]]; then
        DOCKER_PLATFORM="--platform linux/386"
    else
        DOCKER_PLATFORM="--platform linux/amd64"
    fi

    # Build using Docker
    if docker run --rm $DOCKER_PLATFORM \
        -v "$SCRIPT_DIR:/workspace" \
        -w /workspace \
        rust:latest \
        bash -c "rustup target add $target && cargo build --release --target $target" 2>&1; then
        echo -e "${GREEN}✓${NC} Built successfully\n"
    else
        echo -e "${RED}✗${NC} Build failed\n"
        BUILD_ERRORS+=("$target")
    fi
done

# Summary
echo -e "\n${BLUE}=== Build Summary ===${NC}\n"

if [ ${#BUILD_ERRORS[@]} -eq 0 ]; then
    echo -e "${GREEN}All Linux builds completed successfully!${NC}\n"
else
    echo -e "${RED}Some builds failed:${NC}"
    for target in "${BUILD_ERRORS[@]}"; do
        echo -e "  ${RED}✗${NC} ${LINUX_TARGETS[$target]}"
    done
    echo ""
fi

# List built binaries
echo -e "${BLUE}Built binaries:${NC}\n"
for target in "${!LINUX_TARGETS[@]}"; do
    if [[ ! " ${BUILD_ERRORS[@]} " =~ " ${target} " ]]; then
        binary_path="target/${target}/release/scanner"

        if [ -f "$binary_path" ]; then
            size=$(du -h "$binary_path" | cut -f1)
            platform_name="${LINUX_TARGETS[$target]}"

            echo -e "  ${GREEN}✓${NC} ${platform_name}"
            echo -e "    Path: ${binary_path}"
            echo -e "    Size: ${size}"

            # Show file info
            if command -v file &> /dev/null; then
                arch_info=$(file "$binary_path" | cut -d: -f2-)
                echo -e "    Info:${arch_info}"
            fi
            echo ""
        fi
    fi
done

# Create distribution packages
echo -e "${BLUE}Creating distribution packages...${NC}\n"
DIST_DIR="dist"
mkdir -p "$DIST_DIR"

for target in "${!LINUX_TARGETS[@]}"; do
    if [[ ! " ${BUILD_ERRORS[@]} " =~ " ${target} " ]]; then
        binary_path="target/${target}/release/scanner"

        if [ -f "$binary_path" ]; then
            platform_name="${LINUX_TARGETS[$target]}"
            archive_name="scanner-v${VERSION}-${platform_name}"

            tar -czf "${DIST_DIR}/${archive_name}.tar.gz" -C "target/${target}/release" "scanner"
            echo -e "${GREEN}✓${NC} Created ${archive_name}.tar.gz"
        fi
    fi
done

echo -e "\n${GREEN}Docker Linux builds complete!${NC}"
echo -e "Distribution packages are in: ${DIST_DIR}/"

if [ ${#BUILD_ERRORS[@]} -ne 0 ]; then
    exit 1
fi
