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
cd "$PROJECT_ROOT/demo-app"

# Check if demo-app exists
if [ ! -f "Cargo.toml" ]; then
    log_error "demo-app not found"
    log_info "Run: bash $PROJECT_ROOT/scripts/setup-demo.sh"
    exit 1
fi

# Check for trunk
if ! command -v trunk &> /dev/null; then
    log_error "trunk not found. Install with: cargo install trunk"
    exit 1
fi

# Check if demo data exists
if [ ! -f "assets/demo-data/romance-2d-umap.json" ]; then
    log_warn "Demo data not found"
    log_info "Run: bash $PROJECT_ROOT/scripts/create-demo-data.sh"
    log_info "(Requires API server running on port 8080)"
fi

# Check if port is in use
if lsof -ti:8888 > /dev/null 2>&1; then
    log_error "Port 8888 is already in use"
    log_info "Kill existing process with: lsof -ti:8888 | xargs kill"
    exit 1
fi

log_success "Starting demo server..."
log_info "URL: http://127.0.0.1:8888/umap/"
log_info "     or: http://localhost:8888/umap/"
log_warn "IMPORTANT: The trailing slash is REQUIRED"
log_info ""
log_info "Press Ctrl+C to stop"
log_info ""

exec trunk serve --port 8888
