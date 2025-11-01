#!/bin/bash
# Test the demo app using automated browser testing
# Requires: scripts/serve-demo.sh to be running
# Can be run from anywhere

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}ℹ ${NC}$1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

log_info "Testing demo at http://127.0.0.1:8888/umap/"

# Check if server is running
if ! curl -s "http://127.0.0.1:8888/umap/" > /dev/null 2>&1; then
    log_error "Demo server not responding on port 8888"
    log_info "Start it with: bash $SCRIPT_DIR/serve-demo.sh"
    exit 1
fi

log_success "Server is running"

# Check if index.html is accessible
if curl -s "http://127.0.0.1:8888/umap/" | grep -q "UMAP Text Visualizer"; then
    log_success "index.html is accessible"
else
    log_error "index.html missing or incorrect"
    exit 1
fi

# Check if demo data is accessible
if curl -s "http://127.0.0.1:8888/assets/demo-data/romance-2d-umap.json" > /dev/null 2>&1; then
    log_success "Demo data is accessible"
else
    log_error "Demo data not found at /assets/demo-data/"
    exit 1
fi

# Check if WASM file is accessible
WASM_FILE=$(curl -s "http://127.0.0.1:8888/umap/" | grep -o 'umap-demo-[a-f0-9]*_bg\.wasm' | head -1)
if [ -n "$WASM_FILE" ]; then
    if curl -s "http://127.0.0.1:8888/$WASM_FILE" > /dev/null 2>&1; then
        log_success "WASM file is accessible: $WASM_FILE"
    else
        log_error "WASM file not accessible: $WASM_FILE"
        exit 1
    fi
else
    log_error "WASM file reference not found in HTML"
    exit 1
fi

log_success "All tests passed!"
log_info "Demo is ready at: http://127.0.0.1:8888/umap/"
