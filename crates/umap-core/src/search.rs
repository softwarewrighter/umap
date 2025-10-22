use anyhow::Result;

use crate::{db::Db, embedding::cosine_similarity, types::ChunkRecord};

#[derive(Debug, Clone)]
pub struct ScoredChunk {
    pub record: ChunkRecord,
    pub score: f32,
}

pub fn top_k_by_cosine(db: &Db, query_vec: &[f32], k: usize) -> Result<Vec<ScoredChunk>> {
    let mut all = db.all_chunks()?;
    let mut scored: Vec<ScoredChunk> = all
        .drain(..)
        .map(|rec| {
            let score = cosine_similarity(&rec.vector, query_vec);
            ScoredChunk { record: rec, score }
        })
        .collect();
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    scored.truncate(k);
    Ok(scored)
}
