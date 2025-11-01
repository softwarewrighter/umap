#!/bin/bash
# Build everything: main app + demo
# Can be run from anywhere

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() { echo -e "${BLUE}ℹ ${NC}$1"; }
log_success() { echo -e "${GREEN}✓${NC} $1"; }
log_error() { echo -e "${RED}✗${NC} $1"; }
log_warn() { echo -e "${YELLOW}⚠${NC} $1"; }

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

log_info "Project root: $PROJECT_ROOT"
cd "$PROJECT_ROOT"

# Verify Cargo.toml exists
if [ ! -f "Cargo.toml" ]; then
    log_error "Could not find Cargo.toml at $PROJECT_ROOT"
    exit 1
fi

log_info "Building workspace (release mode)..."
if cargo build --release; then
    log_success "Workspace built successfully"
else
    log_error "Workspace build failed"
    exit 1
fi

log_info "Building demo app..."
cd "$PROJECT_ROOT/demo-app"

# Check if demo-app exists
if [ ! -f "Cargo.toml" ]; then
    log_error "demo-app not found. Run: bash scripts/setup-demo.sh"
    exit 1
fi

# Check if demo data exists (should be committed to repo)
if [ ! -f "assets/demo-data/romance-2d-umap.json" ]; then
    log_warn "Demo data not found"
    log_info "The demo data files should be committed to the repo."
    log_info "To generate new demo data, run: bash scripts/create-demo-data.sh"
    log_info "(Requires API server running on port 8080)"
fi

# Check for wasm32 target
if ! rustup target list | grep -q "wasm32-unknown-unknown (installed)"; then
    log_warn "wasm32 target not installed. Installing..."
    rustup target add wasm32-unknown-unknown
fi

# Check for trunk
if ! command -v trunk &> /dev/null; then
    log_error "trunk not found. Install with: cargo install trunk"
    exit 1
fi

log_info "Capturing build metadata..."
BUILD_GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_HOST=$(hostname 2>/dev/null || echo "unknown")
BUILD_TIMESTAMP=$(date -u +"%Y-%m-%d %H:%M:%S UTC" 2>/dev/null || echo "unknown")

log_info "Git SHA: $BUILD_GIT_SHA"
log_info "Build host: $BUILD_HOST"
log_info "Build timestamp: $BUILD_TIMESTAMP"

log_info "Building demo with Trunk (release mode)..."
# Set environment variables for Rust compilation
export BUILD_GIT_SHA
export BUILD_HOST
export BUILD_TIMESTAMP

if trunk build --release; then
    log_success "Demo app built successfully"
else
    log_error "Demo app build failed"
    exit 1
fi

# Recreate .nojekyll for GitHub Pages (Trunk deletes it during build)
log_info "Recreating .nojekyll for GitHub Pages..."
touch "$PROJECT_ROOT/docs/.nojekyll"

log_success "All builds complete!"
log_info "Main app binary: $PROJECT_ROOT/target/release/umap-cli"
log_info "Demo output: $PROJECT_ROOT/docs/"
