use anyhow::{Result, anyhow};
use linfa::dataset::DatasetBase;
use linfa::prelude::{Fit, Transformer};
use linfa_reduction::Pca;
use ndarray::Array2;
use rand::prelude::*;

pub trait Reducer {
    fn reduce(&self, data: &Array2<f32>, dims: usize) -> Result<Array2<f32>>;
}

pub struct PcaReducer;

impl Reducer for PcaReducer {
    fn reduce(&self, data: &Array2<f32>, dims: usize) -> Result<Array2<f32>> {
        if !(1..=3).contains(&dims) {
            return Err(anyhow!("dims must be 1..=3"));
        }
        // PCA expects f64 internally; convert
        let n = data.nrows();
        let d = data.ncols();
        let mut data64 = Array2::<f64>::zeros((n, d));
        for i in 0..n {
            for j in 0..d {
                data64[(i, j)] = data[(i, j)] as f64;
            }
        }
        let ds = DatasetBase::from(data64);
        let model = Pca::params(dims)
            .fit(&ds)
            .map_err(|e| anyhow!("PCA fit failed: {e}"))?;
        let transformed = model.transform(ds);
        let x = transformed.records;
        let mut out = Array2::<f32>::zeros((n, dims));
        for i in 0..n {
            for j in 0..dims {
                out[(i, j)] = x[(i, j)] as f32;
            }
        }
        Ok(out)
    }
}

pub fn reduce_default(data: &Array2<f32>, dims: usize) -> Result<Array2<f32>> {
    let reducer = PcaReducer;
    reducer.reduce(data, dims)
}

// ----------------- Minimal UMAP implementation (educational) -----------------

#[derive(Debug, Clone)]
pub struct UmapParams {
    pub n_neighbors: usize,
    pub n_epochs: usize,
    pub min_dist: f32,
    pub spread: f32,
    pub learning_rate: f32,
    pub negative_sample_rate: usize,
    pub set_op_mix_ratio: f32,
    pub repulsion_strength: f32,
    pub random_state: u64,
}

impl Default for UmapParams {
    fn default() -> Self {
        Self {
            n_neighbors: 15,
            n_epochs: 200,
            min_dist: 0.1,
            spread: 1.0,
            learning_rate: 1.0,
            negative_sample_rate: 5,
            set_op_mix_ratio: 1.0,
            repulsion_strength: 1.0,
            random_state: 42,
        }
    }
}

pub fn umap_reduce_cosine(
    data: &Array2<f32>,
    dims: usize,
    params: &UmapParams,
) -> Result<Array2<f32>> {
    if !(2..=3).contains(&dims) {
        return Err(anyhow!("dims must be 2 or 3"));
    }
    let n = data.nrows();
    if n == 0 {
        return Ok(Array2::<f32>::zeros((0, dims)));
    }
    if n <= 2 {
        return Ok(Array2::<f32>::zeros((n, dims)));
    }
    let _d = data.ncols();

    // 1) kNN (cosine distance)
    let k = params.n_neighbors.min(n.saturating_sub(1)).max(2);
    let (indices, dists) = knn_cosine(data, k);

    // 2) Smooth kNN distances (rho_i, sigma_i)
    let (rhos, sigmas) = smooth_knn_distances(&dists, k as f32, 64, 1.0, 1.0);

    // 3) Fuzzy simplicial set weights p_ij
    let mut rows = Vec::<usize>::new();
    let mut cols = Vec::<usize>::new();
    let mut vals = Vec::<f32>::new();
    for i in 0..n {
        for nn in 0..k {
            let j = indices[(i, nn)];
            if i == j {
                continue;
            }
            let d_ij = dists[(i, nn)];
            let pij = if d_ij - rhos[i] > 0.0 {
                (-((d_ij - rhos[i]) / (sigmas[i] + 1e-8))).exp()
            } else {
                1.0
            };
            rows.push(i);
            cols.push(j);
            vals.push(pij);
        }
    }
    // Symmetrize with set_op_mix_ratio (fuzzy union)
    let (rows, cols, vals) = symmetrize(n, rows, cols, vals, params.set_op_mix_ratio);

    // 4) Initialize embedding
    let mut y = random_init(n, dims, params.random_state);

    // 5) Optimize via SGD on cross-entropy between high-dim fuzzy set and low-dim
    let (a, b) = find_ab_params(params.spread, params.min_dist);
    let opt_params = OptimizeParams {
        a,
        b,
        n_epochs: params.n_epochs,
        learning_rate: params.learning_rate,
        negative_sample_rate: params.negative_sample_rate,
        repulsion_strength: params.repulsion_strength,
        seed: params.random_state,
    };
    optimize_layout(&mut y, &rows, &cols, &vals, &opt_params);

    Ok(y)
}

fn knn_cosine(data: &Array2<f32>, k: usize) -> (Array2<usize>, Array2<f32>) {
    let n = data.nrows();
    let d = data.ncols();
    let mut inds = Array2::<usize>::zeros((n, k));
    let mut dists = Array2::<f32>::zeros((n, k));
    // Precompute norms
    let mut norms = vec![0f32; n];
    for i in 0..n {
        let mut s = 0.0;
        for j in 0..d {
            s += data[(i, j)] * data[(i, j)];
        }
        norms[i] = s.sqrt();
    }
    for i in 0..n {
        let mut all: Vec<(usize, f32)> = (0..n)
            .filter(|&j| j != i)
            .map(|j| {
                let mut dot = 0.0f32;
                for c in 0..d {
                    dot += data[(i, c)] * data[(j, c)];
                }
                let denom = (norms[i] * norms[j]).max(1e-8);
                let cos = dot / denom;
                let dist = 1.0 - cos; // cosine distance
                (j, dist)
            })
            .collect();
        all.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        for nn in 0..k {
            inds[(i, nn)] = all[nn].0;
            dists[(i, nn)] = all[nn].1;
        }
    }
    (inds, dists)
}

fn smooth_knn_distances(
    dists: &Array2<f32>,
    k: f32,
    n_iter: usize,
    local_connectivity: f32,
    bandwidth: f32,
) -> (Vec<f32>, Vec<f32>) {
    let n = dists.nrows();
    let mut rhos = vec![0f32; n];
    let mut sigmas = vec![0f32; n];
    let target = (k.ln()) * bandwidth;
    for i in 0..n {
        // rho_i: distance to the nearest neighbor at local_connectivity
        let lc = local_connectivity.max(1.0);
        let idx = (lc.floor() as usize).saturating_sub(1);
        let rho = dists[(i, idx)].max(0.0);

        // binary search for sigma
        let mut lo = 0.0f32;
        let mut hi = 1.0f32;
        // Increase hi until sum >= target
        for _ in 0..8 {
            let val = smooth_knn_fn(dists, i, rho, hi);
            if val > target {
                break;
            }
            hi *= 2.0;
        }
        let mut sigma = hi;
        for _ in 0..n_iter {
            let mid = 0.5 * (lo + hi);
            let val = smooth_knn_fn(dists, i, rho, mid);
            if (val - target).abs() < 1e-5 {
                sigma = mid;
                break;
            }
            if val > target {
                hi = mid;
            } else {
                lo = mid;
            }
            sigma = mid;
        }
        rhos[i] = rho;
        sigmas[i] = sigma.max(1e-8);
    }
    (rhos, sigmas)
}

fn smooth_knn_fn(dists: &Array2<f32>, i: usize, rho: f32, sigma: f32) -> f32 {
    let k = dists.ncols();
    let mut psum = 0.0f32;
    for nn in 0..k {
        let d = dists[(i, nn)];
        let v = if d > rho {
            (-((d - rho) / sigma)).exp()
        } else {
            1.0
        };
        psum += v;
    }
    psum
}

fn symmetrize(
    n: usize,
    rows: Vec<usize>,
    cols: Vec<usize>,
    vals: Vec<f32>,
    mix: f32,
) -> (Vec<usize>, Vec<usize>, Vec<f32>) {
    // Combine p_ij and p_ji as p = mix*(p_ij + p_ji - p_ij*p_ji) + (1-mix)*0.5*(p_ij + p_ji)
    use std::collections::HashMap;
    let mut map: HashMap<(usize, usize), f32> = HashMap::new();
    for (r, c, v) in rows
        .into_iter()
        .zip(cols.into_iter())
        .zip(vals.into_iter())
        .map(|((r, c), v)| (r, c, v))
    {
        let key = (r, c);
        *map.entry(key).or_insert(0.0) = v;
    }
    let mut out_map: HashMap<(usize, usize), f32> = HashMap::new();
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let pij = *map.get(&(i, j)).unwrap_or(&0.0);
            let pji = *map.get(&(j, i)).unwrap_or(&0.0);
            if pij == 0.0 && pji == 0.0 {
                continue;
            }
            let prod = pij * pji;
            let fuzzy = pij + pji - prod;
            let avg = 0.5 * (pij + pji);
            let v = mix * fuzzy + (1.0 - mix) * avg;
            out_map.insert((i, j), v);
        }
    }
    let mut rows = Vec::with_capacity(out_map.len());
    let mut cols = Vec::with_capacity(out_map.len());
    let mut vals = Vec::with_capacity(out_map.len());
    for ((i, j), v) in out_map.into_iter() {
        rows.push(i);
        cols.push(j);
        vals.push(v);
    }
    (rows, cols, vals)
}

fn random_init(n: usize, dims: usize, seed: u64) -> Array2<f32> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut y = Array2::<f32>::zeros((n, dims));
    for i in 0..n {
        for d in 0..dims {
            y[(i, d)] = rng.r#gen::<f32>() * 1e-3 - 5e-4;
        }
    }
    y
}

fn find_ab_params(spread: f32, min_dist: f32) -> (f32, f32) {
    // Approximate solution adapted from UMAP reference implementation behavior
    // Values near (a=1.5769, b=0.8951) when spread=1.0, min_dist=0.1
    let mut a = 1.5769f32;
    let mut b = 0.8951f32;
    // Simple heuristic adjustment based on min_dist and spread
    let md = min_dist.max(1e-3);
    let s = spread.max(1e-3);
    a *= 1.0 / s;
    b *= (0.1 / md).powf(0.5).clamp(0.3, 3.0);
    (a, b)
}

struct OptimizeParams {
    a: f32,
    b: f32,
    n_epochs: usize,
    learning_rate: f32,
    negative_sample_rate: usize,
    repulsion_strength: f32,
    seed: u64,
}

fn optimize_layout(
    y: &mut Array2<f32>,
    rows: &[usize],
    cols: &[usize],
    vals: &[f32],
    params: &OptimizeParams,
) {
    let n_edges = rows.len();
    if n_edges == 0 {
        return;
    }
    let n = y.nrows();
    let dims = y.ncols();
    let mut rng = StdRng::seed_from_u64(params.seed);

    for _epoch in 0..params.n_epochs {
        for e in 0..n_edges {
            let i = rows[e];
            let j = cols[e];
            let w = vals[e];
            let grad = edge_gradient(y, i, j, params.a, params.b, params.repulsion_strength);
            for d in 0..dims {
                y[(i, d)] -= params.learning_rate * w * grad.0[d];
                y[(j, d)] += params.learning_rate * w * grad.0[d];
            }

            // Negative samples
            for _ in 0..params.negative_sample_rate {
                let jn = rng.gen_range(0..n);
                if jn == i {
                    continue;
                }
                let gradn =
                    negative_gradient(y, i, jn, params.a, params.b, params.repulsion_strength);
                for d in 0..dims {
                    y[(i, d)] += params.learning_rate * gradn[d];
                    y[(jn, d)] -= params.learning_rate * gradn[d];
                }
            }
        }
    }
}

fn edge_gradient(
    y: &Array2<f32>,
    i: usize,
    j: usize,
    a: f32,
    b: f32,
    _repulsion_strength: f32,
) -> (Vec<f32>, f32) {
    let dims = y.ncols();
    let mut diff = vec![0f32; dims];
    let mut dist2 = 0.0f32;
    for d in 0..dims {
        let v = y[(i, d)] - y[(j, d)];
        diff[d] = v;
        dist2 += v * v;
    }
    let dist = dist2.sqrt();
    // Attractive force derivative for 1/(1 + a d^{2b})
    let q = 1.0 / (1.0 + a * dist.powf(2.0 * b));
    let grad_coeff = 2.0 * b * a * dist.powf(2.0 * b - 2.0) * q * q;
    let mut grad = vec![0f32; dims];
    for d in 0..dims {
        grad[d] = grad_coeff * diff[d];
    }
    (grad, q)
}

fn negative_gradient(
    y: &Array2<f32>,
    i: usize,
    j: usize,
    a: f32,
    b: f32,
    repulsion_strength: f32,
) -> Vec<f32> {
    let dims = y.ncols();
    let mut diff = vec![0f32; dims];
    let mut dist2 = 0.0f32;
    for d in 0..dims {
        let v = y[(i, d)] - y[(j, d)];
        diff[d] = v;
        dist2 += v * v;
    }
    let dist = dist2.sqrt();
    let qq = 1.0 / (1.0 + a * dist.powf(2.0 * b));
    let grad_coeff = repulsion_strength * 2.0 * b / (1e-3 + dist) * qq * (1.0 - qq);
    let mut grad = vec![0f32; dims];
    for d in 0..dims {
        grad[d] = grad_coeff * diff[d];
    }
    grad
}
