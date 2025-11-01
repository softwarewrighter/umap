# Design

This document outlines key design choices and interfaces.

Embedding
- Approach: feature hashing on tokens to fixed `dim` (default 512).
- Tokenization: lowercase, split on non-word, filter short tokens, optional stopword removal.
- Normalization: L2-normalize vectors for cosine similarity.

Chunking
- Split on paragraph boundaries; fallback to sentence windows (e.g., 5–8 sentences) if paragraphs are too large.
- Store `source` and `chunk_index` for traceability.

Storage (SQLite)
- Table `chunks(id, source, chunk_index, text, dim, vector, created_at)`.
- `vector` stores contiguous `f32` values as a BLOB. Endianness: little-endian.
- All reads/writes go through `umap_core::db` for consistency.

Search
- Compute query embedding; iterate rows; compute cosine similarity; keep top-k in a max-heap.
- For small corpora this is fast enough and vastly simpler than building indices.

Reduction Abstraction
- Trait `Reducer` with `reduce(points: &Array2<f32>, dims: usize) -> Array2<f32>`.
- Implementation: `PcaReducer` (always available) using `linfa-reduction`.
- Optional: `UmapReducer` behind `feature = "umap"` if/when a Rust UMAP crate is added. API-compatible with `Reducer`.

HTTP API
- `GET /api/search`: query params `query`, `k` (default 30), `dims` (2 or 3).
- Response includes points and metadata ready for plotting.

Frontend (Yew + Plotly)
- Component `App`: search input, 2D/3D toggle, submit button, plot area, list of neighbor snippets.
- Uses `plotly_yew::Plotly` to render scatter (`Scatter` or `Scatter3D`).

Build/Run
- Build web via Trunk: `trunk build --release --dist crates/umap-web/dist`.
- Serve via CLI: `umap-cli serve --db data.db --static-dir crates/umap-web/dist`.

