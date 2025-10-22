#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT_DIR"

DB_PATH=${DB_PATH:-data.db}
TOKENS_PER_CHUNK=${TOKENS_PER_CHUNK:-500}
OVERLAP=${OVERLAP:-100}
BIN="./target/release/umap-cli"

if [ ! -x "$BIN" ]; then
  echo "[ingest-all] Release binary not found, using debug binary..." >&2
  BIN="./target/debug/umap-cli"
fi

if [ ! -x "$BIN" ]; then
  echo "[ingest-all] No binary found. Please run: cargo build --release -p umap-cli" >&2
  exit 1
fi

echo "[ingest-all] Starting batch ingestion to $DB_PATH"
echo "[ingest-all] Settings: tokens_per_chunk=$TOKENS_PER_CHUNK, overlap=$OVERLAP"
echo ""

# Ingest docs (smaller chunks for documentation)
if [ -d "docs" ] && [ -n "$(ls -A docs/*.md 2>/dev/null)" ]; then
  echo "[ingest-all] Ingesting documentation files from docs/*.md"
  for file in docs/*.md; do
    echo "  → $(basename "$file")"
    "$BIN" ingest --db "$DB_PATH" --file "$file" \
      --tokens-per-chunk 300 --overlap 50 --dim 512
  done
  echo ""
fi

# Ingest fiction texts (larger chunks for narrative)
if [ -d "tmp" ] && [ -n "$(ls -A tmp/*.txt 2>/dev/null)" ]; then
  echo "[ingest-all] Ingesting fiction texts from tmp/*.txt"
  for file in tmp/*.txt; do
    echo "  → $(basename "$file")"
    "$BIN" ingest --db "$DB_PATH" --file "$file" \
      --tokens-per-chunk "$TOKENS_PER_CHUNK" --overlap "$OVERLAP" --dim 512
  done
  echo ""
fi

# Show final count
TOTAL=$("$BIN" search --db "$DB_PATH" --query "test" --k 1 2>&1 | grep "complete; total rows" | tail -1 | grep -o '[0-9]*$' || echo "unknown")
echo "[ingest-all] ✅ Batch ingestion complete!"
echo "[ingest-all] Total chunks in database: Check with CLI"
echo ""
echo "To view, run: bash scripts/serve.sh"
