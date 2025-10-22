#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v rustup >/dev/null 2>&1; then
  echo "[build] rustup not found. Please install Rust toolchain (https://rustup.rs)." >&2
  exit 1
fi

if ! command -v trunk >/dev/null 2>&1; then
  echo "[build] Trunk not found. Install with: cargo install trunk" >&2
  exit 1
fi

# Ensure wasm target is present (needed by Trunk)
if ! rustup target list --installed | grep -q '^wasm32-unknown-unknown$'; then
  echo "[build] Adding wasm32-unknown-unknown target (rustup target add wasm32-unknown-unknown)"
  rustup target add wasm32-unknown-unknown
fi

echo "[build] Building web UI (WASM) with Trunk..."
pushd "$ROOT_DIR/crates/umap-web" >/dev/null
rm -rf dist
# Use debug mode to avoid wasm-opt parsing errors
trunk build --dist dist
popd >/dev/null

echo "[build] Building workspace (CLI + core)..."
cargo build --release -p umap-cli

echo "[build] Done. UI in crates/umap-web/dist, binary at target/release/umap-cli"
