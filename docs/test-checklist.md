# Test Checklist

Use this checklist to validate the app end-to-end after a fresh build.

Build and Serve
- Build UI + app: `scripts/build.sh`
- Serve API + UI: `scripts/serve.sh` (defaults: DB `data.db`, `127.0.0.1:8080`)
- Verify `/` and SPA fallback:
  - `curl -i http://127.0.0.1:8080/` → 200 OK (HTML)
  - `curl -i http://127.0.0.1:8080/any/route` → 200 OK (HTML)

Ingestion
- UI: Choose one or more `.txt` files, set tokens/chunk and overlap, click “Ingest Files”.
  - UI: Status shows “Ingested …”; if errors, a red banner shows details.
  - Server logs: `ingest completed` with `filename`, `chunks`, `elapsed_ms`.
- API: `curl -i -X POST /api/ingest_text` with JSON body
  - Returns 200 JSON: `{ filename, chunks, total_rows }`

Search + Visualization
- UI: Enter query, select method (UMAP or PCA), toggle 2D/3D, set k; click Search.
  - UI: Status shows `Fetched N points` or a red error banner on failure.
  - UI: Plotly scatter visible; hover shows snippet and score.
  - Server logs: `search completed` with `method`, `dims`, `k`, `n_neighbors`, `min_dist`, `points`, `elapsed_ms`.
- API: `curl -i '/api/search?query=q&k=50&dims=3&method=umap'`
  - Returns 200 JSON: `{ points: [ { id, source, chunk_index, score, text_preview, x, y, z? }, ... ] }`

Logging Expectations
- Ingest: server logs filename, number of chunks, and timing; UI logs ingest POST and displays confirmation.
- Search: server logs UMAP params, dims, k, timing, and point count; UI logs the GET URL and renders results.

Troubleshooting
- Blank UI: ensure `scripts/build.sh` ran; `scripts/serve.sh` path points to `crates/umap-web/dist`.
- Plotly not defined: ensure Plotly CDN script is present in `crates/umap-web/index.html` and served before app.
- MIME type (text/html) for JS: static server returned HTML for JS path; confirm using the Rust CLI server, not a generic file server.
- No points returned: try a more common query or lower k; verify ingestion logged chunks; for small datasets, UMAP returns zeros but still responds.

