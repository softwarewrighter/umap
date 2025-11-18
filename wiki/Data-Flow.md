# Data Flow

This page details the data flow patterns and sequences for key operations in the UMAP Text Visualizer.

## Overview

The system has three primary workflows:
1. **Ingestion** - Processing text files into embeddings
2. **Search** - Finding similar chunks and visualizing results
3. **Runtime Ingestion** - Adding text via API

## Ingestion Flow

### CLI Ingest Command

```mermaid
sequenceDiagram
    participant User
    participant CLI as umap-cli
    participant Chunker as chunk.rs
    participant Embedder as embedding.rs
    participant DB as db.rs
    participant SQLite

    User->>CLI: ingest --file text.txt --db data.db
    CLI->>Chunker: chunk_text(content, tokens_per_chunk, overlap)
    Chunker->>Chunker: Split paragraphs/sentences
    Chunker-->>CLI: Vec<ChunkMetadata>

    loop For each chunk
        CLI->>Embedder: embed_text(chunk.text, dim)
        Embedder->>Embedder: Tokenize (lowercase, split)
        Embedder->>Embedder: Feature hashing to fixed dim
        Embedder->>Embedder: L2 normalize
        Embedder-->>CLI: Vec<f32>

        CLI->>DB: insert_chunk(metadata, vector)
        DB->>SQLite: INSERT INTO chunks
        SQLite-->>DB: chunk_id
        DB-->>CLI: Ok(id)
    end

    CLI-->>User: Ingested N chunks
```

**Key Steps:**

1. User invokes CLI with file path and database
2. Text file is read into memory
3. Chunker splits text into overlapping segments
4. Each chunk is:
   - Tokenized (lowercase, filter short words)
   - Hashed to fixed-dimension vector
   - L2-normalized
   - Stored in SQLite with metadata
5. Progress is reported to user

## Search Flow

### Search Query Processing

```mermaid
sequenceDiagram
    participant User as Browser User
    participant WebUI as Yew App
    participant API as Axum Server
    participant Search as search.rs
    participant Embedder as embedding.rs
    participant Reducer as reduction.rs
    participant DB as db.rs
    participant SQLite

    User->>WebUI: Enter query + click Search
    WebUI->>API: GET /api/search?query=...&k=30&dims=3&method=umap

    API->>Embedder: embed_text(query, dim)
    Embedder->>Embedder: Tokenize + hash + normalize
    Embedder-->>API: query_vector

    API->>Search: search(query_vector, k)
    Search->>DB: get_all_chunks()
    DB->>SQLite: SELECT * FROM chunks
    SQLite-->>DB: Vec<(id, metadata, vector)>
    DB-->>Search: chunks

    Search->>Search: Compute cosine similarity for all
    Search->>Search: Keep top-K in max-heap
    Search-->>API: Vec<SearchResult>

    API->>Reducer: reduce(vectors, dims, method)
    alt method == "umap"
        Reducer->>Reducer: UMAP gradient descent
    else method == "pca"
        Reducer->>Reducer: PCA via linfa
    end
    Reducer-->>API: coordinates (2D or 3D)

    API->>API: Merge coordinates with metadata
    API-->>WebUI: JSON response

    WebUI->>WebUI: Parse points
    WebUI->>WebUI: Render Plotly scatter
    WebUI-->>User: Interactive visualization
```

**Key Steps:**

1. User enters query in web UI
2. Web UI sends HTTP GET to `/api/search`
3. Server embeds query using same hasher
4. Sequential scan computes cosine similarity for all chunks
5. Top-K results kept in max-heap
6. Results passed to dimensionality reducer
7. UMAP or PCA produces 2D/3D coordinates
8. Coordinates merged with metadata (text, score, source)
9. JSON response sent to client
10. Client renders interactive Plotly scatter plot

### Cosine Similarity Calculation

```mermaid
graph TD
    Query[Query Vector] --> Normalize1[L2 Normalized]
    DB[(Database)] --> Vectors[All Chunk Vectors]
    Vectors --> Normalize2[Already L2 Normalized]

    Normalize1 --> Dot[Dot Product]
    Normalize2 --> Dot

    Dot --> Similarity[Cosine Similarity]
    Similarity --> Heap[Max-Heap Top-K]
    Heap --> Results[Search Results]
```

Since all vectors are L2-normalized, cosine similarity reduces to a simple dot product:

```
cosine_sim(a, b) = dot(a, b)  // when ||a|| = ||b|| = 1
```

## Runtime Ingestion Flow

### POST API Ingestion

```mermaid
sequenceDiagram
    participant Client
    participant API as Axum Server
    participant Chunker as chunk.rs
    participant Embedder as embedding.rs
    participant DB as db.rs
    participant SQLite

    Client->>API: POST /api/ingest_text
    Note over Client,API: JSON: {text, source, dim, tokens_per_chunk, overlap}

    API->>Chunker: chunk_text(text, tokens_per_chunk, overlap)
    Chunker-->>API: Vec<ChunkMetadata>

    loop For each chunk
        API->>Embedder: embed_text(chunk.text, dim)
        Embedder-->>API: Vec<f32>

        API->>DB: insert_chunk(metadata, vector)
        DB->>SQLite: INSERT INTO chunks
        SQLite-->>DB: Ok(id)
    end

    API-->>Client: JSON {chunk_count, chunk_ids}
```

**Key Steps:**

1. Client sends POST with JSON payload
2. Server chunks the text
3. Each chunk is embedded and stored
4. Server responds with chunk count and IDs

## Dimensionality Reduction Flow

### UMAP Algorithm

```mermaid
graph TD
    Input[High-Dim Vectors] --> Graph[Construct k-NN Graph]
    Graph --> Fuzzy[Compute Fuzzy Simplicial Set]
    Fuzzy --> Init[Initialize Low-Dim Coordinates]
    Init --> Optimize[Gradient Descent Optimization]

    Optimize --> Attractive[Attractive Forces]
    Optimize --> Repulsive[Repulsive Forces]

    Attractive --> Update[Update Coordinates]
    Repulsive --> Update

    Update --> Converge{Converged?}
    Converge -->|No| Optimize
    Converge -->|Yes| Output[2D/3D Coordinates]
```

**UMAP Parameters:**

- `n_neighbors` - Size of local neighborhood (default 15)
- `min_dist` - Minimum distance between points (default 0.1)
- `n_epochs` - Optimization iterations (default 200)
- `learning_rate` - Step size for gradient descent (default 1.0)

See [[Architecture#UMAP Parameters]] for full parameter list.

### PCA Algorithm

```mermaid
graph TD
    Input[High-Dim Vectors] --> Center[Center Data]
    Center --> Cov[Compute Covariance Matrix]
    Cov --> Eigen[Eigenvalue Decomposition]
    Eigen --> Select[Select Top K Components]
    Select --> Project[Project Data]
    Project --> Output[2D/3D Coordinates]
```

PCA is deterministic and fast, but captures only linear relationships.

## Frontend Update Flow

### Plotly Rendering

```mermaid
sequenceDiagram
    participant Component as Yew Component
    participant State as Component State
    participant Plotly as Plotly.js (WASM binding)
    participant DOM

    Component->>State: Receive API response
    State->>State: Parse JSON to SearchResponse
    State->>State: Update internal state

    Component->>Component: Re-render triggered
    Component->>Plotly: Create Scatter/Scatter3D trace
    Note over Component,Plotly: Extract x, y, z, text, hovertext

    Plotly->>Plotly: Build plotly.js config
    Plotly->>DOM: Render plot to <div>
    DOM-->>Component: Plot rendered

    Note over Component: User can interact (zoom, rotate, hover)
```

**Data Transformation:**

```
API Response          Plotly Format
-------------         -------------
{                     {
  points: [             x: [1.2, 3.4, ...],
    {x: 1.2,            y: [5.6, 7.8, ...],
     y: 5.6,            z: [9.0, 1.1, ...],  // if 3D
     z: 9.0,            text: ["chunk1", "chunk2", ...],
     text: "...",       hovertext: ["score: 0.95", ...]
     score: 0.95}     }
  ]
}
```

## Error Handling Flow

### Search Error Handling

```mermaid
graph TD
    Request[API Request] --> Validate{Valid?}
    Validate -->|No| BadRequest[400 Bad Request]
    Validate -->|Yes| Embed[Embed Query]

    Embed --> EmbedOk{Success?}
    EmbedOk -->|No| ServerError1[500 Internal Error]
    EmbedOk -->|Yes| Search[Database Search]

    Search --> SearchOk{Success?}
    SearchOk -->|No| ServerError2[500 Internal Error]
    SearchOk -->|Yes| Reduce[Dimensionality Reduction]

    Reduce --> ReduceOk{Success?}
    ReduceOk -->|No| ServerError3[500 Internal Error]
    ReduceOk -->|Yes| Success[200 OK + JSON]
```

**Error Categories:**

- **400 Bad Request** - Invalid parameters (missing query, invalid dims)
- **500 Internal Server Error** - Database errors, reduction failures
- **503 Service Unavailable** - Database connection failures

## Performance Considerations

### Sequential Scan Performance

```mermaid
graph LR
    subgraph "Current: O(n) Search"
        A1[Query] --> B1[Scan All N Vectors]
        B1 --> C1[Compute N Similarities]
        C1 --> D1[Heap Top-K]
    end

    subgraph "Future: O(log n) with Index"
        A2[Query] --> B2[HNSW Index]
        B2 --> C2[Approximate K Neighbors]
        C2 --> D2[Refine if Needed]
    end
```

**Trade-offs:**

Current (sequential scan):
- Simple implementation
- Exact results
- Works for small datasets (<100k chunks)
- No index maintenance

Future (approximate search):
- Complex implementation
- Approximate results (tunable)
- Scales to millions of chunks
- Index build/update overhead

## Data Persistence

### Write Path

```mermaid
graph LR
    App[Application] --> DB[db.rs]
    DB --> Serialize[Serialize f32 vector]
    Serialize --> Blob[BLOB bytes]
    Blob --> SQL[SQLite INSERT]
    SQL --> File[data.db file]
```

### Read Path

```mermaid
graph LR
    File[data.db file] --> SQL[SQLite SELECT]
    SQL --> Blob[BLOB bytes]
    Blob --> Deserialize[Deserialize to f32 vector]
    Deserialize --> DB[db.rs]
    DB --> App[Application]
```

**Vector Serialization:**

```rust
// Write
let bytes: Vec<u8> = vector.iter()
    .flat_map(|f| f.to_le_bytes())
    .collect();

// Read
let floats: Vec<f32> = bytes.chunks_exact(4)
    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
    .collect();
```

All vectors use little-endian encoding for portability.

## Related Pages

- [[Architecture]] - System architecture overview
- [[umap-core]] - Core library implementation
- [[umap-cli]] - CLI and API implementation
- [[umap-web]] - Frontend implementation

## External Resources

- [Axum Request Lifecycle](https://docs.rs/axum/latest/axum/#routing) - Axum routing and handlers
- [SQLite BLOB Performance](https://www.sqlite.org/intern-v-extern-blob.html) - BLOB storage recommendations
- [UMAP Algorithm](https://arxiv.org/abs/1802.03426) - Original UMAP paper with algorithm details
