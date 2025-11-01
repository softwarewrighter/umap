#!/bin/bash
# Serve the main UMAP application with API server
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
cd "$PROJECT_ROOT"

# Check if binary exists
if [ ! -f "target/release/umap-cli" ]; then
    log_error "Binary not found. Run: bash scripts/build-all.sh"
    exit 1
fi

# Check if database exists
if [ ! -f "data.db" ]; then
    log_warn "Database not found at data.db"
    log_info "You may need to ingest data first:"
    log_info "  ./target/release/umap-cli ingest --db data.db --file <your-file.txt>"
fi

# Check if web UI is built
if [ ! -d "crates/umap-web/dist" ]; then
    log_warn "Web UI not built at crates/umap-web/dist"
    log_info "Building now..."
    bash "$SCRIPT_DIR/build-all.sh"
fi

# Check if port is in use
if lsof -ti:8080 > /dev/null 2>&1; then
    log_error "Port 8080 is already in use"
    log_info "Kill existing process with: lsof -ti:8080 | xargs kill"
    exit 1
fi

log_success "Starting UMAP server on http://127.0.0.1:8080"
log_info "Press Ctrl+C to stop"
log_info ""

exec ./target/release/umap-cli serve \
    --db data.db \
    --static-dir crates/umap-web/dist \
    --addr 127.0.0.1:8080
