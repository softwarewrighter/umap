# Architecture Overview

Goal: A Rust CLI-driven system to ingest text into a SQLite-backed vector store, perform search + UMAP dimensionality reduction, and serve an interactive Yew (WASM) web UI with 2D/3D visualizations via Plotly.

Components
- `umap-core` (library):
  - Text chunking and preprocessing
  - Hashing-based text embedding (feature hashing) with cosine normalization
  - SQLite persistence layer storing text chunks and embedding vectors
  - Search with cosine similarity (sequential scan for simplicity)
  - Dimensionality reduction abstraction with UMAP (if available) and PCA fallback

- `umap-cli` (binary):
  - CLI commands: `ingest`, `serve`, `search`
  - `ingest`: chunk + embed text, persist into SQLite
  - `serve`: Axum HTTP server exposing JSON API and static files for Yew app
  - `search`: quick CLI nearest-neighbor summary

- `umap-web` (wasm + Yew):
  - UI for entering a search query and toggling 2D/3D
  - Fetches `/api/search?query=...&k=...&dims=...`
  - Renders scatter via `plotly_yew` with tooltips showing source snippets

Data Model (SQLite)
- Table `chunks`:
  - `id INTEGER PRIMARY KEY`
  - `source TEXT` (e.g., filename)
  - `chunk_index INTEGER`
  - `text TEXT`
  - `dim INTEGER` (embedding dimension)
  - `vector BLOB` (serialized `f32` array of length `dim`)
  - `created_at TEXT` (ISO 8601)

APIs (Axum)
- `GET /api/search?query=<q>&k=<n>&dims=2|3`: returns JSON with:
  - `query_vec`: the embedded query vector metadata (optional)
  - `points`: array of `{ id, source, chunk_index, text_preview, score, x, y, z? }`
  - Backend computes neighbors, then reduces to the desired dims.

UMAP/PCA Flow
1. Collect candidate vectors (top-k NN by cosine; or sample all for small demos).
2. Reduce vectors (UMAP preferred; PCA fallback) to 2D/3D.
3. Return coordinates with metadata for plotting.

Serving Frontend
- Build `umap-web` with Trunk to `crates/umap-web/dist`.
- `umap-cli serve --static-dir crates/umap-web/dist` serves the SPA along with the API.

Notes
- For larger corpora or speedups, later integrate a SQLite vector extension (e.g., sqlite-vss) to offload k-NN indexing, or adopt approximate nearest neighbor libs.
- Embedding can be swapped to a model-based approach later; the current hashing embedder keeps the demo self-contained and portable.

