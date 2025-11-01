# Product Requirements Document (PRD)

## Goal
Provide an end-to-end, local, demonstrable system showing how UMAP can visualize high-dimensional text embeddings in 2D/3D, with search-driven exploration.

## Users
- ML practitioners and engineers exploring dimensionality reduction.
- Educators demonstrating UMAP on textual data.

## Use Cases
- Ingest a public domain `.txt` file, chunk into segments, embed, and store.
- Enter a search query; see the closest chunked passages and their positions in a 2D/3D projection.
- Compare 2D vs. 3D layouts; tweak k (neighbors returned) quickly.

## Core Features
- CLI ingestion: `umap-cli ingest --db data.db --file book.txt`
- CLI serve: `umap-cli serve --db data.db --static-dir crates/umap-web/dist`
- CLI search: `umap-cli search --db data.db --query "..." --k 20`
- Web UI: query input, 2D/3D toggle, interactive Plotly scatter with tooltips.
- Backend reduction: UMAP (preferred) with PCA fallback.

## Non-Goals (v1)
- Production-grade ANN indexing or vector extensions.
- Distributed serving or multi-user auth.
- Parametric UMAP training.

## Success Criteria
- Can ingest a text file (<10MB) in <30s on a laptop.
- Search returns top-20 neighbors in <1s for small corpora.
- Interactive visualization renders in <2s for up to ~1,000 points.

## Constraints
- Pure Rust, local runtime.
- SQLite for storage; sequential cosine similarity ok for v1.
- WASM front-end with Yew + Plotly.

## Future Enhancements
- Swap in vector DB extensions (sqlite-vss) for scalable k-NN.
- Integrate transformer embeddings via local models.
- Add parameter controls (n_neighbors, min_dist) in UI.

