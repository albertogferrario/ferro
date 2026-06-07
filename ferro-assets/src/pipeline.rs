//! Pipeline builder, Transform trait, and map_matching helper.

use crate::{Asset, ContentType, Error};

/// A transform operates over the entire asset collection.
///
/// Files whose [`ContentType`] is not in the transform's accepted set must be
/// returned unchanged (byte-identical passthrough — the crate's core guarantee).
///
/// Implementors are [`Send`] + [`Sync`] so a `Pipeline` can be sent to
/// `tokio::task::spawn_blocking`.
pub trait Transform: Send + Sync {
    /// Apply this transform to `assets`, returning the (possibly mutated) set.
    ///
    /// Files not accepted by this transform MUST be returned byte-identical.
    /// On any error, return `Err` — the pipeline propagates it immediately and
    /// produces no partial output.
    fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error>;
}

/// Convenience helper for transforms that work file-by-file on accepted types.
///
/// Files whose [`ContentType`] is not in `accepted` are passed through
/// with no allocation. The iterator short-circuits on the first `Err` so
/// no partial output is ever produced.
pub fn map_matching<F>(
    assets: Vec<Asset>,
    accepted: &[ContentType],
    mut f: F,
) -> Result<Vec<Asset>, Error>
where
    F: FnMut(Asset) -> Result<Asset, Error>,
{
    assets
        .into_iter()
        .map(|a| {
            if accepted.contains(&a.content_type) {
                f(a)
            } else {
                Ok(a)
            }
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Ordered composition of [`Transform`]s over a heterogeneous asset set.
///
/// Transforms are applied in insertion order. Any `Err` from a transform
/// immediately returns from [`Pipeline::run`] — no partial output set is produced.
///
/// `Pipeline::run()` is a **blocking** call. Wrap it in
/// `tokio::task::spawn_blocking` when calling from an async context.
pub struct Pipeline {
    transforms: Vec<Box<dyn Transform>>,
}

impl Pipeline {
    /// Create an empty pipeline.
    pub fn new() -> Self {
        Self { transforms: vec![] }
    }

    /// Append a transform to the end of the chain.
    ///
    /// Transforms are executed in insertion order.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, t: impl Transform + 'static) -> Self {
        self.transforms.push(Box::new(t));
        self
    }

    /// Run all transforms in order over `assets`.
    ///
    /// All-or-nothing: any `Err` from a transform returns immediately with no
    /// partial `Vec<Asset>`.
    ///
    /// This is a **blocking** call. Wrap in `tokio::task::spawn_blocking` when
    /// calling from an async context to avoid stalling the async executor during
    /// CPU-bound transform work (HTML minification, image encoding, etc.).
    pub fn run(&self, assets: Vec<Asset>) -> Result<Vec<Asset>, Error> {
        let mut current = assets;
        for transform in &self.transforms {
            current = transform.run(current)?;
        }
        Ok(current)
    }
}

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}
