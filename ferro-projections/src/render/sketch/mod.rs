//! Cross-modality sketch renderers for intent vocabulary validation.
//!
//! # Research sketch — not stable API
//!
//! See `docs/research/comp-05-cross-modality-vocabulary-sketch.md` for the
//! full 7-intent x 3-modality analysis and v14.0 implications.

pub(crate) mod cli;
pub(crate) mod mobile;
pub(crate) mod voice;

#[allow(unused_imports)]
pub(crate) use cli::CliSummaryRenderer;
#[allow(unused_imports)]
pub(crate) use mobile::MobileCardRenderer;
#[allow(unused_imports)]
pub(crate) use voice::VoiceRenderer;
