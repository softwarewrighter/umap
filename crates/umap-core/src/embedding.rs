pub struct HasherEmbedder {
    dim: usize,
    // Optional seeded projections for signed hashing; here we do simple hashing trick
}

impl HasherEmbedder {
    pub fn new(dim: usize) -> Self {
        Self { dim }
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    pub fn embed(&self, text: &str) -> Vec<f32> {
        let tokens = tokenize(text);
        let mut vec = vec![0f32; self.dim];
        for t in tokens {
            let h = fxhash(&t);
            let idx = (h % (self.dim as u64)) as usize;
            // signed hashing using lowest bit
            let sign = if (h & 1) == 0 { 1.0 } else { -1.0 };
            vec[idx] += sign;
        }
        l2_normalize(vec)
    }
}

pub fn tokenize(text: &str) -> Vec<String> {
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '\'')
        .map(|s| s.trim_matches('\''))
        .filter(|s| !s.is_empty())
        .filter(|s| s.len() > 1)
        .map(|s| s.to_string())
        .collect()
}

fn l2_normalize(mut v: Vec<f32>) -> Vec<f32> {
    let norm = (v.iter().map(|x| (*x as f64) * (*x as f64)).sum::<f64>()).sqrt() as f32;
    if norm > 0.0 {
        for x in &mut v {
            *x /= norm;
        }
    }
    v
}

fn fxhash(s: &str) -> u64 {
    // A simple 64-bit hash; not cryptographic. Fowler–Noll–Vo (FNV-1a)
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for i in 0..a.len() {
        dot += a[i] * b[i];
        na += a[i] * a[i];
        nb += b[i] * b[i];
    }
    let denom = (na.sqrt() * nb.sqrt()).max(1e-8);
    dot / denom
}
