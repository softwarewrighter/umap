#!/usr/bin/env bash
set -euo pipefail

DB_PATH=${DB_PATH:-data.db}
ADDR=${ADDR:-127.0.0.1:8080}
STATIC_DIR=${STATIC_DIR:-crates/umap-web/dist}

if [ ! -f "$STATIC_DIR/index.html" ]; then
  echo "[serve] $STATIC_DIR/index.html not found. Run scripts/build.sh first." >&2
  exit 1
fi

echo "[serve] Serving UI from $STATIC_DIR and API on http://$ADDR"
BIN="./target/release/umap-cli"
if [ ! -x "$BIN" ]; then
  echo "[serve] Release binary not found, attempting debug binary..." >&2
  BIN="./target/debug/umap-cli"
fi
exec "$BIN" serve --db "$DB_PATH" --static-dir "$STATIC_DIR" --addr "$ADDR"
