#!/bin/bash
set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Cross-Compilation Test ===${NC}\n"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

echo -e "Current platform: ${YELLOW}${OS}-${ARCH}${NC}\n"

if [ "$OS" = "darwin" ]; then
    if ! command -v rustup &> /dev/null; then
        echo -e "${YELLOW}rustup not found${NC}"
        echo -e "You have Rust installed via Homebrew, which doesn't support cross-compilation targets.\n"
        echo -e "${GREEN}To enable cross-compilation:${NC}"
        echo -e "1. Install rustup:"
        echo -e "   ${YELLOW}curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh${NC}"
        echo -e "2. Add cross-compilation targets:"
        echo -e "   ${YELLOW}rustup target add x86_64-apple-darwin aarch64-apple-darwin${NC}"
        echo -e "3. Run this script again\n"
        echo -e "${YELLOW}Building for native architecture only:${NC}"
        cargo build --release
        ls -lh target/release/scanner
        exit 0
    fi
    
    echo -e "${GREEN}macOS detected - can cross-compile for both architectures${NC}\n"
    
    TARGETS=("x86_64-apple-darwin" "aarch64-apple-darwin")
    
    for TARGET in "${TARGETS[@]}"; do
        echo -e "${GREEN}Testing ${TARGET}...${NC}"
        
        # Check if target is installed
        if rustup target list --installed | grep -q "$TARGET"; then
            echo -e "  ${GREEN}✓${NC} Target installed"
        else
            echo -e "  ${YELLOW}⚠${NC} Target not installed"
            echo -e "  Installing ${TARGET}..."
            rustup target add "$TARGET"
        fi
        
        # Try to build
        echo -e "  Building..."
        if cargo build --release --target "$TARGET" 2>&1 | tail -5; then
            BINARY="target/${TARGET}/release/scanner"
            SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
            echo -e "  ${GREEN}✓${NC} Build successful (${SIZE})"
            
            # Show binary info
            file "$BINARY"
        else
            echo -e "  ${RED}✗${NC} Build failed"
        fi
        echo ""
    done
    
    echo -e "${GREEN}Summary:${NC}"
    echo "Available binaries:"
    find target -name scanner -type f -path "*/release/*" -exec ls -lh {} \; | awk '{print "  " $9 " (" $5 ")"}'
    
elif [ "$OS" = "linux" ]; then
    echo -e "${YELLOW}Linux detected${NC}\n"
    
    if command -v cross &> /dev/null; then
        echo -e "${GREEN}cross tool found - can cross-compile${NC}\n"
        
        TARGETS=("x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu")
        
        for TARGET in "${TARGETS[@]}"; do
            echo -e "${GREEN}Testing ${TARGET}...${NC}"
            
            if cross build --release --target "$TARGET" 2>&1 | tail -5; then
                BINARY="target/${TARGET}/release/scanner"
                SIZE=$(ls -lh "$BINARY" | awk '{print $5}')
                echo -e "  ${GREEN}✓${NC} Build successful (${SIZE})"
                file "$BINARY"
            else
                echo -e "  ${RED}✗${NC} Build failed"
            fi
            echo ""
        done
    else
        echo -e "${YELLOW}cross tool not found${NC}"
        echo -e "Install with: ${YELLOW}cargo install cross${NC}"
        echo ""
        echo "Without cross, you can only build for your native architecture:"
        cargo build --release
        ls -lh target/release/scanner
    fi
else
    echo -e "${YELLOW}Unsupported OS for cross-compilation${NC}"
    echo "Building for native architecture only:"
    cargo build --release
    ls -lh target/release/scanner
fi

echo ""
echo -e "${GREEN}Done!${NC}"
