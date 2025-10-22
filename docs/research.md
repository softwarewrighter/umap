# UMAP Research Notes

Overview: Uniform Manifold Approximation and Projection (UMAP) is a non-linear dimension reduction technique based on manifold learning and topological data analysis. It constructs a weighted k-nearest-neighbor graph in the high-dimensional space, optimizes a low-dimensional embedding by minimizing the cross-entropy between fuzzy simplicial sets, and preserves both local and some global structure. It is widely used as an alternative to t-SNE due to speed and better preservation of global relationships.

- Foundations: Uses Riemannian geometry assumptions (uniformly distributed on a Riemannian manifold) and fuzzy topological representations (fuzzy simplicial sets). The main steps are: (1) compute neighbor graph with adaptive radii (via k-NN and smooth local distances); (2) build a fuzzy union of local simplicial sets; (3) optimize a low-dim layout by minimizing cross-entropy between high- and low-dim fuzzy sets.
- Similarity metric: Often cosine or Euclidean. For text embeddings, cosine is typical. UMAP supports arbitrary metrics with k-NN search.
- Hyperparameters: `n_neighbors` controls the balance between local vs. global structure; `min_dist` controls how tightly points can cluster in the embedding; `n_components` is target dimension (2 or 3 common); `metric` defines distance in the original space; `learning_rate`, `negative_sample_rate`, and `n_epochs` affect optimization speed and quality.
- Strengths: Fast on large datasets (especially with approximate kNN), preserves local neighborhoods, stable global structure, supports 2D/3D and arbitrary metrics. Works well as a visualization tool after embedding (e.g., text embeddings) and as a preprocessing step for downstream modeling.
- Limitations: Non-parametric by default (no direct transform for new points unless a parametric variant is used), stochastic optimization can yield slightly different layouts per run, choice of hyperparameters matters, and small datasets can overcluster.

Comparisons
- vs. PCA: PCA is linear and preserves maximum variance; fast and deterministic. UMAP is nonlinear and better preserves manifold structure and local neighborhoods.
- vs. t-SNE: t-SNE excels at local cluster separation but often loses global relationships and can be slower. UMAP tends to be faster, with better preservation of global structure, and provides parameters (`min_dist`, `n_neighbors`) that are intuitive for tuning cluster tightness and scale.

Usage for Text
- Text → Embedding (e.g., transformer-based embeddings or simpler TF-IDF/hash embeddings) → UMAP to 2D/3D → Visualize clusters and neighborhoods.
- For interactive demos: search a query, embed it, find nearest neighbors in the corpus, and visualize those neighbors plus context in 2D/3D.

Recommended Defaults (starting points)
- `n_neighbors`: 15–50 (start at 15 for local structure; increase for more global continuity)
- `min_dist`: 0.05–0.3 (smaller gives tighter clusters)
- `metric`: cosine for text; euclidean for continuous vectors
- `n_components`: 2 (scatter plot) or 3 (3D scatter)

Notes on Implementation in Rust
- For a pure-Rust demo, you can compute k-NN in Rust and use `linfa-reduction` for PCA fallback if a UMAP crate is not available. Some community crates provide UMAP implementations; parametric UMAP typically requires training a neural net (not necessary for this demo).
- For small datasets (hundreds to a few thousands of points), a simple O(n) scan per query with cosine similarity is sufficient and easy to implement.

