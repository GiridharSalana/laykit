#!/bin/bash
# Comprehensive test runner for LayKit
# Run from tests/ directory or project root

set -e

# Detect if we're in tests/ or project root
if [ -f "Cargo.toml" ]; then
    # We're in project root
    PROJECT_ROOT="."
    TESTS_DIR="tests"
elif [ -f "../Cargo.toml" ]; then
    # We're in tests/ directory
    PROJECT_ROOT=".."
    TESTS_DIR="."
else
    echo "Error: Cannot find project root (Cargo.toml not found)"
    exit 1
fi

cd "$PROJECT_ROOT"

echo "================================"
echo "LayKit Comprehensive Test Suite"
echo "================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Step 1: Rust tests
echo "Step 1: Running Rust tests..."
echo "------------------------------"
if cargo test --all-features; then
    echo -e "${GREEN}✓ Rust tests passed${NC}"
else
    echo -e "${RED}✗ Rust tests failed${NC}"
    exit 1
fi
echo ""

# Step 2: Build release binary
echo "Step 2: Building release binary..."
echo "-----------------------------------"
if cargo build --release; then
    echo -e "${GREEN}✓ Build successful${NC}"
else
    echo -e "${RED}✗ Build failed${NC}"
    exit 1
fi
echo ""

# Step 3: Check for gdstk
echo "Step 3: Checking for gdstk..."
echo "------------------------------"
if python3 -c "import gdstk" 2>/dev/null; then
    echo -e "${GREEN}✓ gdstk found${NC}"
    GDSTK_AVAILABLE=true
else
    echo -e "${YELLOW}⚠ gdstk not found - skipping validation tests${NC}"
    echo "  Install with: pip install gdstk"
    GDSTK_AVAILABLE=false
fi
echo ""

# Step 4: Run gdstk validation (if available)
if [ "$GDSTK_AVAILABLE" = true ]; then
    echo "Step 4: Running gdstk cross-validation..."
    echo "------------------------------------------"
    if python3 tests/gdstk_validation.py; then
        echo -e "${GREEN}✓ Validation tests passed${NC}"
    else
        echo -e "${RED}✗ Validation tests failed${NC}"
        exit 1
    fi
    echo ""
fi

# Summary
echo "================================"
echo -e "${GREEN}All tests completed successfully!${NC}"
echo "================================"
echo ""
echo "Test Summary:"
echo "  ✓ Rust unit tests"
echo "  ✓ Rust integration tests"
echo "  ✓ Rust doc tests"
echo "  ✓ Release build"
if [ "$GDSTK_AVAILABLE" = true ]; then
    echo "  ✓ gdstk validation tests"
else
    echo "  ⚠ gdstk validation (skipped)"
fi
echo ""

