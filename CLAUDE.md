# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

UMAP Text Visualizer is a Rust (2024 edition) CLI and web UI demonstrating dimensionality reduction (UMAP/PCA) of text embeddings with interactive 2D/3D visualization. It consists of three workspace crates:

- `umap-core`: Core library with chunking, embedding, SQLite persistence, search, and dimensionality reduction
- `umap-cli`: CLI tool and Axum HTTP server (commands: `ingest`, `serve`, `search`)
- `umap-web`: Yew (WASM) + Plotly web frontend

## Build Commands

Build the entire workspace:
```bash
cargo build
cargo build --release
```

Build specific crate:
```bash
cargo build -p umap-core
cargo build -p umap-cli
cargo build -p umap-web
```

Build the web UI (requires Trunk and wasm32 target):
```bash
# Install prerequisites if needed:
# cargo install trunk
# rustup target add wasm32-unknown-unknown

cd crates/umap-web
trunk build --release --dist dist
```

## Testing

Run all tests:
```bash
cargo test
```

Run tests for a specific crate:
```bash
cargo test -p umap-core
cargo test -p umap-cli
```

Run a single test by name:
```bash
cargo test <test_name>
```

## Running the Application

Ingest text data:
```bash
./target/debug/umap-cli ingest --db data.db --file /path/to/file.txt
# Optional params: --dim (default 512), --tokens-per-chunk (default 1000), --overlap (default 300)
```

Run the HTTP server:
```bash
./target/debug/umap-cli serve --db data.db --static-dir crates/umap-web/dist --addr 127.0.0.1:8080
```

Perform CLI search:
```bash
./target/debug/umap-cli search --db data.db --query "your search phrase" --k 20
```

## Architecture

### Core Components

**umap-core modules:**
- `chunk.rs`: Text chunking with token overlap
- `embedding.rs`: Hashing-based text embeddings (feature hashing) with L2 normalization
- `db.rs`: SQLite persistence layer for chunks and embedding vectors
- `search.rs`: Cosine similarity search (sequential scan)
- `reduction.rs`: Dimensionality reduction abstraction (PCA, UMAP)
- `types.rs`: Shared data structures (Point2D, Point3D, etc.)

**umap-cli:**
- Single binary with subcommands (`ingest`, `serve`, `search`)
- Axum HTTP server with two API endpoints:
  - `GET /api/search?query=<q>&k=<n>&dims=2|3&method=umap|pca` (with UMAP params)
  - `POST /api/ingest_text` for runtime text ingestion
- Static file serving for the Yew frontend

**umap-web:**
- Yew (WASM) SPA with Plotly scatter plot visualization
- Uses `gloo-net` for HTTP requests
- Supports 2D/3D toggle and adjustable UMAP parameters

### Data Model

SQLite table `chunks`:
- `id` (INTEGER PRIMARY KEY)
- `source` (TEXT): filename or identifier
- `chunk_index` (INTEGER): position in source
- `text` (TEXT): the chunk content
- `dim` (INTEGER): embedding dimension
- `vector` (BLOB): serialized f32 array (little-endian)
- `created_at` (TEXT): ISO 8601 timestamp

### Embedding Strategy

Uses feature hashing (hashing trick) on tokens to create fixed-dimension vectors:
1. Tokenize text (lowercase, split on non-word, filter short tokens)
2. Hash tokens to dimension indices
3. L2-normalize resulting vectors for cosine similarity

This approach is lightweight and self-contained (no external models required).

### Dimensionality Reduction

Two methods available:
- **PCA**: Always available via `linfa-reduction`
- **UMAP**: Implemented via custom gradient descent algorithm in `reduction.rs`

The `Reducer` trait abstracts the reduction interface. Future UMAP crates can be integrated via feature flags.

## Development Notes

### Adding a New Module to umap-core

1. Create the module file in `crates/umap-core/src/`
2. Add `pub mod module_name;` to `lib.rs`
3. Optionally re-export key types with `pub use module_name::*;`

### Modifying the HTTP API

The Axum routes are defined in `crates/umap-cli/src/main.rs` in the `cmd_serve` function. Add new routes before the `.with_state()` call.

Query parameters are deserialized via `serde::Deserialize` structs (see `SearchParams` and `IngestTextReq`).

### Updating the Frontend

The Yew app is in `crates/umap-web/src/lib.rs`. After making changes:
1. Rebuild with `trunk build --release --dist dist` from the `crates/umap-web` directory
2. Restart the server to serve the updated static files

### Sequential Scan Search

Current search implementation iterates all rows and computes cosine similarity. For larger datasets, consider:
- SQLite vector extensions (e.g., sqlite-vss)
- External ANN libraries
- Sampling strategies

### UMAP Parameters

The API accepts tunable UMAP parameters:
- `n_neighbors`: neighborhood size (default 15)
- `min_dist`: minimum distance in embedding (default 0.1)
- `spread`: scale of embedded points (default 1.0)
- `n_epochs`: training iterations (default 200)
- `learning_rate`: SGD learning rate (default 1.0)
- `negative_sample_rate`: negatives per positive (default 5)
- `set_op_mix_ratio`: fuzzy set operation mix (default 1.0)
- `repulsion_strength`: strength of repulsive force (default 1.0)
- `random_state`: random seed (default 42)

These are exposed in the API for experimentation.

## Key Design Decisions

1. **Rust 2024 edition**: Uses latest Rust features and idioms
2. **No external embedding models**: Feature hashing keeps the project self-contained and portable
3. **SQLite for storage**: Simple, embedded, and sufficient for moderate-scale demos
4. **WASM frontend**: Yew enables Rust for the entire stack
5. **Monorepo workspace**: Three crates share dependencies and remain tightly coupled

## Documentation

Additional design docs in `docs/`:
- `architecture.md`: Component overview and data flow
- `design.md`: Interface details and design rationale
- `PRD.md`: Product requirements
- `plan.md`: Development milestones and risks
- `research.md`: Background research
