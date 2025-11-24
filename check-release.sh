#!/bin/bash

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}=== Release Readiness Check ===${NC}\n"

READY=true

# Check 1: Git repository
if [ -d .git ]; then
    echo -e "${GREEN}✓${NC} Git repository found"
else
    echo -e "${RED}✗${NC} Not a git repository"
    READY=false
fi

# Check 2: Uncommitted changes
if [ -z "$(git status --porcelain)" ]; then
    echo -e "${GREEN}✓${NC} No uncommitted changes"
else
    echo -e "${RED}✗${NC} You have uncommitted changes:"
    git status --short
    READY=false
fi

# Check 3: GitHub CLI
if command -v gh &> /dev/null; then
    echo -e "${GREEN}✓${NC} GitHub CLI installed"
    
    # Check authentication
    if gh auth status &> /dev/null; then
        echo -e "${GREEN}✓${NC} GitHub CLI authenticated"
    else
        echo -e "${RED}✗${NC} GitHub CLI not authenticated (run: gh auth login)"
        READY=false
    fi
else
    echo -e "${RED}✗${NC} GitHub CLI not installed (install from: https://cli.github.com/)"
    READY=false
fi

# Check 4: Rust toolchain
if command -v cargo &> /dev/null; then
    echo -e "${GREEN}✓${NC} Rust toolchain installed"
else
    echo -e "${RED}✗${NC} Rust toolchain not installed"
    READY=false
fi

# Check 5: Current version
if [ -f Cargo.toml ]; then
    CURRENT_VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
    echo -e "${GREEN}✓${NC} Current version: ${YELLOW}${CURRENT_VERSION}${NC}"
else
    echo -e "${RED}✗${NC} Cargo.toml not found"
    READY=false
fi

# Check 6: Remote repository
if git remote get-url origin &> /dev/null; then
    REMOTE=$(git remote get-url origin)
    echo -e "${GREEN}✓${NC} Remote repository: ${YELLOW}${REMOTE}${NC}"
else
    echo -e "${YELLOW}⚠${NC} No remote repository configured"
fi

# Check 7: Current branch
BRANCH=$(git branch --show-current)
if [ "$BRANCH" = "main" ] || [ "$BRANCH" = "master" ]; then
    echo -e "${GREEN}✓${NC} On main branch: ${YELLOW}${BRANCH}${NC}"
else
    echo -e "${YELLOW}⚠${NC} Not on main branch (current: ${YELLOW}${BRANCH}${NC})"
fi

echo ""
if [ "$READY" = true ]; then
    echo -e "${GREEN}✓ Ready to release!${NC}"
    echo ""
    echo "Run one of:"
    echo "  ./release.sh --patch   # ${CURRENT_VERSION} → $(echo $CURRENT_VERSION | awk -F. '{print $1"."$2"."$3+1}')"
    echo "  ./release.sh --minor   # ${CURRENT_VERSION} → $(echo $CURRENT_VERSION | awk -F. '{print $1"."$2+1".0"}')"
    echo "  ./release.sh --major   # ${CURRENT_VERSION} → $(echo $CURRENT_VERSION | awk -F. '{print $1+1".0.0"}')"
    exit 0
else
    echo -e "${RED}✗ Not ready to release${NC}"
    echo "Fix the issues above and try again"
    exit 1
fi
