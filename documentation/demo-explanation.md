# Demo Visualization Explanation: UMAP vs PCA

## Overview

The demo provides four visualization modes comparing two dimensionality reduction methods (UMAP and PCA) in both 2D and 3D:

1. **Romance (2D UMAP)** - UMAP projection to 2 dimensions
2. **Romance (2D PCA)** - PCA projection to 2 dimensions
3. **Romance (3D UMAP)** - UMAP projection to 3 dimensions
4. **Romance (3D PCA)** - PCA projection to 3 dimensions

## Theoretical Background

### PCA (Principal Component Analysis)

**What it does:**
- Finds orthogonal axes of maximum variance in the data
- Performs **linear** transformation
- First component captures most variance, second captures next most, etc.

**Pros:**
- Fast and deterministic (always same result)
- Well-understood mathematical properties
- Good for data with linear structure
- Preserves global structure and distances
- Works well when variance = importance

**Cons:**
- **Cannot capture non-linear relationships**
- Assumes linear manifold structure
- May miss local neighborhood structure
- Can be dominated by outliers
- First few PCs might not capture semantic meaning

### UMAP (Uniform Manifold Approximation and Projection)

**What it does:**
- Constructs a fuzzy topological representation of high-dimensional data
- Performs **non-linear** manifold learning
- Preserves local neighborhood structure
- Uses stochastic gradient descent

**Pros:**
- **Captures non-linear manifold structure**
- Preserves local neighborhoods (similar points stay together)
- Better for clustering visualization
- Can reveal hidden structure that linear methods miss
- Tunable via multiple hyperparameters

**Cons:**
- Stochastic (different runs can vary slightly due to random_state)
- More computationally expensive than PCA
- Many hyperparameters to tune (n_neighbors, min_dist, etc.)
- Can be sensitive to parameter choices
- Less interpretable axes than PCA

## The Four Demo Views

### 2D Visualizations

**2D PCA:**
- Shows data projected onto the two principal components
- Points spread along axes of maximum variance
- Often appears "organized" or "linear" in structure
- Axes have mathematical meaning (directions of variance)

**2D UMAP:**
- Shows data projected onto 2D manifold preserving local structure
- May show clusters or groupings
- Axes have no inherent meaning (arbitrary rotation)
- Focuses on preserving neighborhoods, not global distances

### 3D Visualizations

**3D PCA:**
- Three principal components (top 3 variance directions)
- Additional dimension can reveal structure hidden in 2D
- Still constrained to linear projections
- Can rotate to explore different angles

**3D UMAP:**
- Three-dimensional manifold embedding
- More freedom to arrange points while preserving neighborhoods
- Can separate clusters more clearly than 2D
- Interactive rotation helps understand structure

## Important Observation: Why PCA Might Look "Better"

### Your Observation is Valid!

If the 2D PCA visualization shows clearer clusters or better separation than 2D UMAP, this is **not necessarily a bug** - it can happen for several legitimate reasons:

### Possible Explanations

#### 1. **Dataset Size (30 points)**
- UMAP is designed for larger datasets (typically 100s-1000s of points)
- With only 30 points, UMAP may not have enough data to learn manifold structure
- PCA can work well even with small datasets
- **Recommendation**: UMAP typically needs more data to shine

#### 2. **Embedding Method (Feature Hashing)**
- This project uses **feature hashing** (hashing trick) for embeddings
- Feature hashing creates relatively **linear** high-dimensional representations
- When the high-dimensional space is already linear, PCA works great!
- True semantic embeddings (like BERT, GPT) have more non-linear structure
- **Key insight**: PCA doing well suggests the feature hashing creates linear structure

#### 3. **UMAP Hyperparameters**
The demo uses these UMAP settings:
```
n_neighbors = 15       # neighborhood size
min_dist = 0.1         # minimum distance in embedding
n_epochs = 200         # training iterations
learning_rate = 1.0    # SGD learning rate
```

For a 30-point dataset, `n_neighbors=15` means each point's neighborhood is **50% of the dataset**, which is quite large. This might cause UMAP to behave more "globally" (similar to PCA).

**Better parameters for small datasets:**
```
n_neighbors = 5-10     # smaller neighborhood
min_dist = 0.01        # tighter clusters
n_epochs = 500         # more training
```

#### 4. **UMAP Implementation**
This project uses a **custom UMAP implementation** (see `crates/umap-core/src/reduction.rs`). While it implements the core algorithm, it may not be as optimized or robust as the reference Python implementation (umap-learn).

**Potential issues:**
- Gradient descent might not be fully converged
- Force calculations might have subtle bugs
- Spectral initialization might differ from reference
- Cross-entropy optimization might need tuning

#### 5. **Data Characteristics**
The demo data is from a romance query on literature text. If:
- The texts don't have strong semantic clusters (all similar romance themes)
- The variance in the data is truly linear
- The chunks are somewhat uniformly distributed

Then PCA's "spread along variance axes" might legitimately be more informative than UMAP's attempt to preserve non-existent clusters.

## When to Use Each Method

### Use PCA When:
- Dataset is small (<100 points)
- You need deterministic, reproducible results
- You want interpretable axes (variance directions)
- Data structure is primarily linear
- Speed is critical
- You need to explain results mathematically

### Use UMAP When:
- Dataset is large (>100 points)
- You suspect non-linear relationships
- You want to visualize clusters
- Preserving local neighborhoods is important
- You're using rich semantic embeddings (BERT, etc.)
- You can afford to tune hyperparameters

## Recommendations for This Project

### Short-term (Current Demo):
1. **Keep both methods** - the comparison is educational!
2. **Add explanation** - note that PCA may work well for feature-hashed embeddings
3. **Tune UMAP parameters** - try smaller `n_neighbors` (5-10)
4. **Add more demo datasets** - show cases where UMAP excels

### Long-term (Future Improvements):
1. **Test with real embeddings** - use sentence-transformers or similar
2. **Larger datasets** - 100+ points to show UMAP's strengths
3. **Benchmark against umap-learn** - compare with reference implementation
4. **Add parameter controls** - let users adjust UMAP hyperparameters
5. **Multiple datasets** - show diverse cases (some favor PCA, some UMAP)

## Conclusion

**Is this a bug?** Probably not a complete bug, but possibly:
- Suboptimal hyperparameters for small datasets
- Implementation details that could be improved
- Natural result of using linear embeddings (feature hashing)

**The real insight:** The fact that PCA shows clear structure suggests your feature hashing creates good linear representations! This isn't bad - it means:
- Your embedding method works
- PCA is the right tool for this embedding space
- UMAP might help more with true semantic embeddings

**Educational value:** The demo successfully shows that method selection matters! Not all data requires non-linear methods. Sometimes the "simpler" linear method (PCA) is actually the right choice.

## Further Reading

- **UMAP Paper**: McInnes, L., Healy, J., & Melville, J. (2018). "UMAP: Uniform Manifold Approximation and Projection for Dimension Reduction"
- **PCA**: Jolliffe, I. T. (2002). "Principal Component Analysis"
- **Comparison**: "Dimensionality Reduction: A Comparative Review" - van der Maaten et al.

## Debug Checklist

If you want to investigate whether UMAP could work better:

- [ ] Try `n_neighbors = 5` (smaller neighborhoods for 30 points)
- [ ] Try `min_dist = 0.01` (tighter clusters)
- [ ] Try `n_epochs = 500` (more optimization)
- [ ] Test with larger dataset (100+ points)
- [ ] Test with real sentence embeddings (not feature hashing)
- [ ] Compare against Python umap-learn on same data
- [ ] Visualize high-dimensional distances vs low-dimensional distances
- [ ] Check if UMAP loss is converging properly
