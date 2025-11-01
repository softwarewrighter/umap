#!/bin/bash
# Create demo data from running API server
# Requires API server on port 8080
# Can be run from anywhere - automatically finds project root

set -e

# Find project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "🔍 Project root: $PROJECT_ROOT"

# Verify API server is running
if ! curl -s "http://127.0.0.1:8080/api/search?query=test&k=1" > /dev/null 2>&1; then
    echo "❌ Error: API server not running on port 8080"
    echo "   Start it with: ./target/release/umap-cli serve --db data.db --static-dir crates/umap-web/dist --addr 127.0.0.1:8080"
    exit 1
fi

echo "📊 Creating demo data..."

DEMO_DATA="$PROJECT_ROOT/demo-app/assets/demo-data"
mkdir -p "$DEMO_DATA"

echo "  Fetching UMAP data..."
curl -s "http://127.0.0.1:8080/api/search?query=love%20romance%20marriage&k=30&dims=2&method=umap&n_neighbors=15&min_dist=0.1" \
  -o "$DEMO_DATA/romance-2d-umap.json"

echo "  Fetching PCA data..."
curl -s "http://127.0.0.1:8080/api/search?query=love%20romance%20marriage&k=30&dims=2&method=pca" \
  -o "$DEMO_DATA/romance-2d-pca.json"

echo ""
echo "✅ Demo data created in $DEMO_DATA/"
ls -lh "$DEMO_DATA/"
