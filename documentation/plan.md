# Plan

Milestones
- Scaffold workspace: core lib, CLI, web, docs.
- Implement chunking + hashing embeddings + SQLite storage.
- Add search API and PCA reduction; optionally wire UMAP later via feature.
- Build Yew UI with Plotly and integrate with API.
- Polish UX, docs, and usage instructions.

Risks
- Rust UMAP crate availability and WASM compatibility.
- Plotly in WASM: ensure `plotly_yew` works with Yew version.
- Large texts: chunking and sequential scan performance.

Mitigations
- Provide PCA fallback.
- Limit k and dataset size in demo; add sampling.
- Defer ANN/indices to future work.

