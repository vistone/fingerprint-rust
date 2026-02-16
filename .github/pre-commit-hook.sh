#!/bin/bash
# Pre-commit hook to enforce code quality
# To install: cp .github/pre-commit-hook.sh .git/hooks/pre-commit && chmod +x .git/hooks/pre-commit

set -e

echo "🔍 Running pre-commit checks..."
echo ""

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗ Cargo is not installed${NC}"
    exit 1
fi

# 1. Check formatting
echo "📝 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo -e "${RED}✗ Code is not formatted properly${NC}"
    echo -e "${YELLOW}Run: cargo fmt --all${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Code formatting OK${NC}"
echo ""

# 2. Run clippy
echo "🔧 Running clippy..."
if ! cargo clippy --workspace --all-targets --all-features -- -D warnings; then
    echo -e "${RED}✗ Clippy found issues${NC}"
    echo -e "${YELLOW}Fix all clippy warnings before committing${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Clippy checks passed${NC}"
echo ""

# 3. Build check
echo "🏗️  Building workspace..."
if ! cargo build --workspace --all-features; then
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Build successful${NC}"
echo ""

# 4. Run tests (quick check)
echo "🧪 Running tests..."
if command -v cargo-nextest &> /dev/null; then
    if ! cargo nextest run --workspace --all-features; then
        echo -e "${RED}✗ Tests failed${NC}"
        exit 1
    fi
else
    if ! cargo test --workspace --all-features; then
        echo -e "${RED}✗ Tests failed${NC}"
        exit 1
    fi
fi
echo -e "${GREEN}✓ All tests passed${NC}"
echo ""

echo -e "${GREEN}✅ All pre-commit checks passed!${NC}"
echo "Ready to commit! 🎉"
