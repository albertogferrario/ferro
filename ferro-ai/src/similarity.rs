/// Compute cosine similarity between two embedding vectors.
///
/// Returns a value in `[-1.0, 1.0]`: `1.0` for identical direction,
/// `0.0` for orthogonal, `-1.0` for opposite direction.
///
/// # Panics
///
/// Panics if either slice is empty, or if the slices have different lengths.
/// These are programmer errors — callers must provide valid, dimension-consistent
/// embeddings from the same model.
///
/// # Zero-magnitude vectors
///
/// A zero-magnitude vector (all components `0.0`) yields `NaN` via `0.0 / 0.0`.
/// This is NOT a panic case (per the locked contract); real embedded text never
/// produces zero-magnitude vectors. Documented, not guarded.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    assert!(!a.is_empty(), "cosine_similarity: empty slice `a`");
    assert!(!b.is_empty(), "cosine_similarity: empty slice `b`");
    assert_eq!(
        a.len(),
        b.len(),
        "cosine_similarity: dimension mismatch ({} vs {})",
        a.len(),
        b.len()
    );
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    dot / (mag_a * mag_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-6;

    #[test]
    fn identical_vectors() {
        let a = vec![1.0f32, 2.0, 3.0];
        let s = cosine_similarity(&a, &a);
        assert!(
            (s - 1.0).abs() < EPSILON,
            "identical: expected 1.0, got {s}"
        );
    }

    #[test]
    fn orthogonal_vectors() {
        let a = vec![1.0f32, 0.0, 0.0];
        let b = vec![0.0f32, 1.0, 0.0];
        let s = cosine_similarity(&a, &b);
        assert!(
            (s - 0.0).abs() < EPSILON,
            "orthogonal: expected 0.0, got {s}"
        );
    }

    #[test]
    fn opposite_vectors() {
        let a = vec![1.0f32, 2.0, 3.0];
        let b = vec![-1.0f32, -2.0, -3.0];
        let s = cosine_similarity(&a, &b);
        assert!(
            (s - (-1.0)).abs() < EPSILON,
            "opposite: expected -1.0, got {s}"
        );
    }

    #[test]
    #[should_panic(expected = "empty slice")]
    fn panics_on_empty() {
        cosine_similarity(&[], &[]);
    }

    #[test]
    #[should_panic(expected = "dimension mismatch")]
    fn panics_on_dim_mismatch() {
        cosine_similarity(&[1.0], &[1.0, 2.0]);
    }
}
