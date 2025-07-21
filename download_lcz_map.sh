#!/bin/bash

# WUDAPT Global LCZ Map Download Script
# Simple wrapper around the Rust binary downloader

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOWNLOADER="$SCRIPT_DIR/target/release/download_wudapt"

echo -e "${BLUE}🌍 WUDAPT Global LCZ Map Download Script${NC}"
echo "========================================="
echo

# Check if downloader binary exists
if [ ! -f "$DOWNLOADER" ]; then
    echo -e "${YELLOW}⚠️  Download binary not found. Building...${NC}"
    echo "Building the downloader binary..."
    
    cd "$SCRIPT_DIR"
    if command -v cargo >/dev/null 2>&1; then
        if [ -n "$LLVM_CONFIG_PATH" ]; then
            cargo build --bin download_wudapt --release
        else
            LLVM_CONFIG_PATH=/opt/homebrew/opt/llvm/bin/llvm-config cargo build --bin download_wudapt --release
        fi
        echo -e "${GREEN}✅ Build complete!${NC}"
        echo
    else
        echo -e "${RED}❌ Error: Cargo not found. Please install Rust.${NC}"
        exit 1
    fi
fi

# Check if file already exists
if [ -f "wudapt_lcz_global.tif" ] && [ "$1" != "--force" ]; then
    echo -e "${GREEN}✅ Global LCZ Map already exists: wudapt_lcz_global.tif${NC}"
    echo -e "${YELLOW}💡 Use --force to re-download${NC}"
    echo
    
    # Show file info
    if command -v ls >/dev/null 2>&1; then
        echo "File information:"
        ls -lh wudapt_lcz_global.tif
        echo
    fi
    
    echo -e "${BLUE}🎯 Ready to use with urban_classifier!${NC}"
    echo
    echo "Usage examples:"
    echo "  Rust: UrbanClassifier::new(\"wudapt_lcz_global.tif\")"
    echo "  Python: urban_classifier.PyUrbanClassifier(\"wudapt_lcz_global.tif\")"
    exit 0
fi

echo "📥 Starting download of WUDAPT Global LCZ Map..."
echo "📊 Expected size: ~4GB"
echo "⏱️  Expected time: 10-60 minutes (depending on connection)"
echo

# Run the downloader with all arguments passed through
"$DOWNLOADER" "$@"

# Check if download was successful
if [ $? -eq 0 ] && [ -f "wudapt_lcz_global.tif" ]; then
    echo
    echo -e "${GREEN}🎉 Download completed successfully!${NC}"
    echo
    
    # Show file info
    if command -v ls >/dev/null 2>&1; then
        echo "File information:"
        ls -lh wudapt_lcz_global.tif
        echo
    fi
    
    echo -e "${BLUE}🎯 Now you can classify coordinates!${NC}"
    echo
    echo "Quick test with Scotland coordinates (57.165, -3.23):"
    echo "  python examples/single_point_demo.py"
    echo
    echo "Or try the full demo:"
    echo "  python examples/python_demo.py"
    
else
    echo
    echo -e "${RED}❌ Download failed. Check the error messages above.${NC}"
    echo
    echo "Troubleshooting:"
    echo "1. Check your internet connection"
    echo "2. Ensure you have ~5GB free disk space"
    echo "3. Try again later if servers are busy"
    echo "4. Use --url to specify a custom download URL"
    exit 1
fi