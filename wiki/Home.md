# UMAP Text Visualizer Wiki

Welcome to the UMAP Text Visualizer wiki! This project is a full-stack Rust application demonstrating dimensionality reduction (UMAP/PCA) of text embeddings with interactive 2D/3D visualizations.

## Quick Links

- [[Architecture]] - System architecture and component overview
- [[Data Flow]] - Sequence diagrams and data flow patterns
- [[umap-core]] - Core library details (chunking, embedding, search, reduction)
- [[umap-cli]] - CLI tool and HTTP server details
- [[umap-web]] - Yew WASM frontend details
- [[Deployment]] - Deployment guide for GitHub Pages and local setups

## Overview

UMAP Text Visualizer is a Rust (2024 edition) CLI and web UI demonstrating:

- **Text chunking** and preprocessing
- **Feature hashing** for text embeddings (no external models required)
- **Vector search** with cosine similarity
- **Dimensionality reduction** using UMAP and PCA
- **Interactive visualization** with 2D/3D scatter plots

### Technology Stack

- **Language**: Pure Rust (2024 edition)
- **Backend**: Axum HTTP server, SQLite database
- **Frontend**: Yew (WASM framework), Plotly.js for visualization
- **Build Tools**: Cargo, Trunk (for WASM builds)

### Workspace Crates

The project is organized as a Cargo workspace with three crates:

1. **umap-core** - Core library ([details](umap-core))
   - Text chunking and preprocessing
   - Hashing-based embeddings with L2 normalization
   - SQLite persistence layer
   - Cosine similarity search
   - UMAP/PCA dimensionality reduction

2. **umap-cli** - CLI and server ([details](umap-cli))
   - Commands: `ingest`, `serve`, `search`
   - Axum HTTP server with JSON API
   - Static file serving for frontend

3. **umap-web** - WASM frontend ([details](umap-web))
   - Yew-based SPA
   - Plotly scatter plots (2D/3D)
   - Real-time search and visualization

## Quick Start

### Prerequisites

```bash
# Install Rust toolchain
rustup target add wasm32-unknown-unknown
cargo install trunk
```

### Build and Run

```bash
# Build entire workspace
cargo build --release

# Ingest text data
./target/release/umap-cli ingest --db data.db --file sample.txt

# Build web UI
cd crates/umap-web
trunk build --release --dist dist

# Run server
./target/release/umap-cli serve --db data.db --static-dir crates/umap-web/dist
```

Visit http://localhost:8080 to use the application.

## Repository Documentation

- [Development Process](../../blob/main/documentation/process.md) - Mandatory TDD process and quality checks
- [Architecture Documentation](../../blob/main/documentation/architecture.md) - Detailed architecture notes
- [Design Documentation](../../blob/main/documentation/design.md) - Design decisions and interfaces
- [Product Requirements](../../blob/main/documentation/PRD.md) - Original requirements
- [Test Checklist](../../blob/main/documentation/test-checklist.md) - Testing guidelines

## Contributing

This project follows a strict Test-Driven Development process. Please read the [Development Process](../../blob/main/documentation/process.md) before contributing.

Key requirements:
- RED/GREEN/REFACTOR testing cycle
- Zero warnings from cargo build, clippy
- All tests must pass
- UI changes require MCP/Playwright verification
- Pure Rust - no JavaScript/TypeScript/Python

## Demo

A live demo is available on GitHub Pages: [UMAP Text Visualizer Demo](https://softwarewrighter.github.io/umap/)

The demo uses pre-computed data and runs entirely in the browser (WASM).
