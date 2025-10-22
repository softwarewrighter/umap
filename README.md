# UMAP Text Visualizer (Rust + Yew)

## Purpose

This project demonstrates how to implement UMAP (Uniform Manifold Approximation and Projection) for dimensionality reduction and use it to visualize high-dimensional data in Rust. Specifically, it shows how to:

1. **Implement UMAP from scratch** using gradient descent and fuzzy set theory for projecting high-dimensional embeddings to 2D/3D space
2. **Visualize text embeddings** generated from ingested text files and chunks
3. **Visualize similarity search results** by projecting query results into 2D/3D scatter plots for interactive exploration

The system ingests text files, chunks them, generates embeddings (using lightweight feature hashing), stores them in SQLite, and provides both CLI and web-based visualization of search results using UMAP or PCA for dimensionality reduction.

**✅ Status: Fully functional!** This implementation demonstrates UMAP and PCA dimensionality reduction with interactive visualization. The system includes color-coded similarity scores, diverse demo data, and works well for educational purposes.

## Features

- **CLI**: ingest text, search, and serve an API + static UI
- **Web UI**: Yew (WASM) + Plotly for interactive 2D/3D scatter plots
- **Embeddings**: lightweight hashing-based vectors (no external models required)
- **Storage**: SQLite (BLOB vectors), sequential cosine similarity search
- **Reduction**: Custom UMAP implementation with PCA fallback

See `docs/` for research, design, architecture, PRD, and plan.

## Quickstart

Prereqs:
- Rust toolchain (1.82+ recommended)
- Trunk for Yew builds: `cargo install trunk` and `rustup target add wasm32-unknown-unknown`

Build UI:
- `cd crates/umap-web`
- `trunk build --release --dist dist`

Ingest text:
- Single file: `./target/debug/umap-cli ingest --db data.db --file /path/to/file.txt`
- Batch ingest all docs + fiction: `bash scripts/ingest-all.sh`
  - Ingests all `docs/*.md` files (documentation, smaller chunks)
  - Ingests all `tmp/*.txt` files (fiction, larger chunks)
  - Currently: **1,581 chunks** from 14 files

Run server:
- `./target/debug/umap-cli serve --db data.db --static-dir crates/umap-web/dist --addr 127.0.0.1:8080`
- Open `http://127.0.0.1:8080` in a browser, enter a query, toggle 2D/3D, adjust k

CLI search (optional):
- `./target/debug/umap-cli search --db data.db --query "your phrase" --k 20`

## Features & Improvements

✅ **Working Features:**
- UMAP 2D/3D visualization (50-80ms for 30 points)
- PCA 2D/3D visualization (9ms for 30 points)
- Color-coded similarity scores (warmer = more similar)
- Interactive Plotly charts with hover tooltips
- File ingestion via CLI and web UI
- Adjustable UMAP parameters (n_neighbors, min_dist, epochs, etc.)
- Diverse demo dataset with 8 topic clusters

✅ **Recent Improvements:**
- Zero compiler warnings (clean code)
- Color-coded visualization showing similarity to query
- Diverse test data highlighting UMAP advantages over PCA
- Fixed build system (debug mode for Trunk)
- Workspace resolver configuration updated

## Demo Guide

**To see UMAP's advantage over PCA:**

1. Ingest data: `bash scripts/ingest-all.sh` (loads docs + fiction)
2. Start the server: `bash scripts/serve.sh`
3. Open http://127.0.0.1:8080
4. Try diverse queries:
   - **Fiction**: `love romance marriage` → Classic literature passages
   - **Tech**: `UMAP reduction dimensionality` → Documentation chunks
   - **Topics**: `basketball soccer tennis` → Sports content
   - **Nature**: `stars galaxies planets` → Space/astronomy
5. Toggle between UMAP and PCA to compare
6. Warmer colors indicate higher similarity to your query
7. With 1,500+ chunks, UMAP's clustering really shines!

**Why UMAP vs PCA?**
- **PCA**: Fast, linear, preserves global variance
- **UMAP**: Preserves local structure, reveals semantic clusters, better for non-linear data

See `IMPROVEMENTS.md` for detailed comparison and `DIAGNOSIS.md` for full testing results.

## Repository Layout
- `crates/umap-core`: chunking, embedding, SQLite, search, reduction
- `crates/umap-cli`: CLI and Axum HTTP server
- `crates/umap-web`: Yew + Plotly web app (built with Trunk)
- `docs/`: research, architecture, PRD, design, plan

## License
MIT or Apache-2.0

