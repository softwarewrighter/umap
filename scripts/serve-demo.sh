#!/bin/bash
# Serve the static demo app (for local testing before GitHub Pages)
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

log_info "Project root: $PROJECT_ROOT"

# Check if docs/ directory exists with built files
if [ ! -f "$PROJECT_ROOT/docs/index.html" ]; then
    log_error "Built demo not found in docs/"
    log_info "Run: bash $PROJECT_ROOT/scripts/build-all.sh"
    exit 1
fi

# Check for basic-http-server
if ! command -v basic-http-server &> /dev/null; then
    log_error "basic-http-server not found"
    log_info "Install with: cargo install basic-http-server"
    exit 1
fi

# Check if port is in use
if lsof -ti:8888 > /dev/null 2>&1; then
    log_error "Port 8888 is already in use"
    log_info "Kill existing process with: lsof -ti:8888 | xargs kill"
    exit 1
fi

log_success "Starting demo server (serving pre-built files from docs/)..."
log_info "URL: http://127.0.0.1:8888/umap/"
log_info "     or: http://localhost:8888/umap/"
log_warn "IMPORTANT: The trailing slash is REQUIRED"
log_info ""
log_info "Press Ctrl+C to stop"
log_info ""

cd "$PROJECT_ROOT/docs"
exec basic-http-server -a 127.0.0.1:8888 .
