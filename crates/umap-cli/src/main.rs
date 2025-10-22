use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result, anyhow};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use clap::{Parser, Subcommand};
use ndarray::Array2;
use serde::Deserialize;
use tower_http::{
    cors::CorsLayer,
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{Level, info};

use umap_core::{
    Db, HasherEmbedder, Point2D, Point3D, UmapParams, chunk_by_token_overlap, reduce_default,
    top_k_by_cosine, umap_reduce_cosine,
};

#[derive(Clone)]
struct AppState {
    db_path: String,
}

#[derive(Parser, Debug)]
#[command(name = "umap-cli", version, about = "UMAP visualization demo CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Ingest a text file into the SQLite vector store
    Ingest {
        #[arg(long, default_value = "data.db")]
        db: PathBuf,
        #[arg(long)]
        file: PathBuf,
        #[arg(long, default_value_t = 512)]
        dim: usize,
        #[arg(long, default_value_t = 1000)]
        tokens_per_chunk: usize,
        #[arg(long, default_value_t = 300)]
        overlap: usize,
    },

    /// Start the HTTP server and optionally serve a static UI directory
    Serve {
        #[arg(long, default_value = "data.db")]
        db: PathBuf,
        #[arg(long, default_value = "127.0.0.1:8080")]
        addr: SocketAddr,
        #[arg(long)]
        static_dir: Option<PathBuf>,
    },

    /// Quick CLI nearest-neighbor search
    Search {
        #[arg(long, default_value = "data.db")]
        db: PathBuf,
        #[arg(long)]
        query: String,
        #[arg(long, default_value_t = 20)]
        k: usize,
        #[arg(long, default_value_t = 512)]
        dim: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .compact()
        .init();

    let cli = Cli::parse();
    match cli.command {
        Commands::Ingest {
            db,
            file,
            dim,
            tokens_per_chunk,
            overlap,
        } => cmd_ingest(db, file, dim, tokens_per_chunk, overlap).await,
        Commands::Serve {
            db,
            addr,
            static_dir,
        } => cmd_serve(db, addr, static_dir).await,
        Commands::Search { db, query, k, dim } => cmd_search(db, query, k, dim).await,
    }
}

async fn cmd_ingest(
    db_path: PathBuf,
    file: PathBuf,
    dim: usize,
    tokens_per_chunk: usize,
    overlap: usize,
) -> Result<()> {
    let db = Db::open(db_path.to_str().unwrap())?;
    let text = fs::read_to_string(&file).with_context(|| format!("read file {:?}", file))?;
    let chunks = chunk_by_token_overlap(&text, tokens_per_chunk, overlap);
    let embedder = HasherEmbedder::new(dim);
    info!("ingesting {} chunks from {:?}", chunks.len(), file);
    for (i, ch) in chunks.iter().enumerate() {
        let v = embedder.embed(ch);
        db.insert_chunk(file.to_string_lossy().as_ref(), i as i64, ch, &v)?;
    }
    info!("ingest complete; total rows = {}", db.count_chunks()?);
    Ok(())
}

async fn cmd_search(db_path: PathBuf, query: String, k: usize, dim: usize) -> Result<()> {
    let db = Db::open(db_path.to_str().unwrap())?;
    let embedder = HasherEmbedder::new(dim);
    let qvec = embedder.embed(&query);
    let top = top_k_by_cosine(&db, &qvec, k)?;
    println!("Top {} results:", top.len());
    for (rank, sc) in top.iter().enumerate() {
        println!(
            "#{} score={:.3} [{}:{}] {}",
            rank + 1,
            sc.score,
            sc.record.source,
            sc.record.chunk_index,
            preview(&sc.record.text, 120)
        );
    }
    Ok(())
}

fn preview(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

#[derive(Debug, Deserialize)]
struct SearchParams {
    query: String,
    #[serde(default = "default_k")]
    k: usize,
    #[serde(default = "default_dims")]
    dims: usize,
    #[serde(default = "default_dim")]
    dim: usize,
    #[serde(default = "default_method")]
    method: String,
    // UMAP params
    #[serde(default = "default_n_neighbors")]
    n_neighbors: usize,
    #[serde(default = "default_min_dist")]
    min_dist: f32,
    #[serde(default = "default_spread")]
    spread: f32,
    #[serde(default = "default_n_epochs")]
    n_epochs: usize,
    #[serde(default = "default_lr")]
    learning_rate: f32,
    #[serde(default = "default_neg_rate")]
    negative_sample_rate: usize,
    #[serde(default = "default_mix")]
    set_op_mix_ratio: f32,
    #[serde(default = "default_repulsion")]
    repulsion_strength: f32,
    #[serde(default = "default_seed")]
    random_state: u64,
}

fn default_k() -> usize {
    30
}
fn default_dims() -> usize {
    2
}
fn default_dim() -> usize {
    512
}
fn default_method() -> String {
    "umap".to_string()
}
fn default_n_neighbors() -> usize {
    15
}
fn default_min_dist() -> f32 {
    0.1
}
fn default_spread() -> f32 {
    1.0
}
fn default_n_epochs() -> usize {
    200
}
fn default_lr() -> f32 {
    1.0
}
fn default_neg_rate() -> usize {
    5
}
fn default_mix() -> f32 {
    1.0
}
fn default_repulsion() -> f32 {
    1.0
}
fn default_seed() -> u64 {
    42
}

async fn cmd_serve(db_path: PathBuf, addr: SocketAddr, static_dir: Option<PathBuf>) -> Result<()> {
    let state = AppState {
        db_path: db_path.to_string_lossy().to_string(),
    };

    let mut app = Router::new()
        .route("/api/search", get(api_search))
        .route("/api/ingest_text", axum::routing::post(api_ingest_text))
        .with_state(state.clone())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    if let Some(dir) = static_dir {
        let index = dir.join("index.html");
        let svc = ServeDir::new(dir.clone())
            .append_index_html_on_directories(true)
            .fallback(ServeFile::new(index));
        app = app.nest_service("/", svc);
    }

    info!("serving on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

async fn api_search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let res = (|| -> Result<axum::response::Response> {
        if params.dims != 2 && params.dims != 3 {
            return Err(anyhow!("dims must be 2 or 3"));
        }
        let embedder = HasherEmbedder::new(params.dim);
        let qvec = embedder.embed(&params.query);
        let db = Db::open(&state.db_path)?;
        let top = top_k_by_cosine(&db, &qvec, params.k)?;
        let mat = Array2::from_shape_vec(
            (top.len(), params.dim),
            top.iter().flat_map(|s| s.record.vector.clone()).collect(),
        )
        .map_err(|e| anyhow!("array shape error: {e}"))?;
        let reduced = if params.method.to_lowercase() == "pca" {
            reduce_default(&mat, params.dims)?
        } else {
            let uparams = UmapParams {
                n_neighbors: params.n_neighbors,
                n_epochs: params.n_epochs,
                min_dist: params.min_dist,
                spread: params.spread,
                learning_rate: params.learning_rate,
                negative_sample_rate: params.negative_sample_rate,
                set_op_mix_ratio: params.set_op_mix_ratio,
                repulsion_strength: params.repulsion_strength,
                random_state: params.random_state,
            };
            umap_reduce_cosine(&mat, params.dims, &uparams)?
        };

        let took = start.elapsed();
        info!(
            method = %params.method,
            dims = params.dims,
            k = params.k,
            n_neighbors = params.n_neighbors,
            min_dist = params.min_dist,
            elapsed_ms = took.as_millis() as u64,
            points = top.len(),
            "search completed"
        );

        if params.dims == 2 {
            let points: Vec<Point2D> = top
                .iter()
                .enumerate()
                .map(|(i, sc)| Point2D {
                    id: sc.record.id,
                    source: sc.record.source.clone(),
                    chunk_index: sc.record.chunk_index,
                    score: sc.score,
                    text_preview: preview(&sc.record.text, 160),
                    x: reduced[(i, 0)],
                    y: reduced[(i, 1)],
                })
                .collect();
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({ "points": points })),
            )
                .into_response())
        } else {
            let points: Vec<Point3D> = top
                .iter()
                .enumerate()
                .map(|(i, sc)| Point3D {
                    id: sc.record.id,
                    source: sc.record.source.clone(),
                    chunk_index: sc.record.chunk_index,
                    score: sc.score,
                    text_preview: preview(&sc.record.text, 160),
                    x: reduced[(i, 0)],
                    y: reduced[(i, 1)],
                    z: reduced[(i, 2)],
                })
                .collect();
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({ "points": points })),
            )
                .into_response())
        }
    })();

    match res {
        Ok(resp) => resp,
        Err(e) => {
            info!(error = %e, "search error");
            let body = serde_json::json!({ "error": e.to_string() });
            (StatusCode::BAD_REQUEST, Json(body)).into_response()
        }
    }
}

#[derive(Deserialize)]
struct IngestTextReq {
    filename: String,
    content: String,
    #[serde(default = "default_tokens_per_chunk")]
    tokens_per_chunk: usize,
    #[serde(default = "default_overlap")]
    overlap: usize,
    #[serde(default = "default_dim")]
    dim: usize,
}

fn default_tokens_per_chunk() -> usize {
    1000
}
fn default_overlap() -> usize {
    300
}

async fn api_ingest_text(
    State(state): State<AppState>,
    Json(body): Json<IngestTextReq>,
) -> impl IntoResponse {
    let start = std::time::Instant::now();
    let res = (|| -> Result<_> {
        let chunks = chunk_by_token_overlap(&body.content, body.tokens_per_chunk, body.overlap);
        let embedder = HasherEmbedder::new(body.dim);
        let db = Db::open(&state.db_path)?;
        for (i, ch) in chunks.iter().enumerate() {
            let v = embedder.embed(ch);
            db.insert_chunk(&body.filename, i as i64, ch, &v)?;
        }
        let took = start.elapsed();
        info!(filename = %body.filename, chunks = chunks.len(), elapsed_ms = took.as_millis() as u64, "ingest completed");
        Ok(serde_json::json!({
            "filename": body.filename,
            "chunks": chunks.len(),
            "total_rows": db.count_chunks()?,
        }))
    })();
    match res {
        Ok(json) => (StatusCode::OK, Json(json)).into_response(),
        Err(e) => {
            info!(error = %e, "ingest error");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response()
        }
    }
}
