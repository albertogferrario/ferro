//! Digital wallet pass issuance (Apple .pkpass + Google Wallet) for the Ferro framework.
//!
//! See the crate-level docs in the [README](https://crates.io/crates/ferro-wallet) and
//! the design spec at `docs/superpowers/specs/2026-05-11-ferro-wallet-crate.md` in the
//! ferro repo for the public API surface.

pub mod apple;
pub mod config;
pub mod error;
pub mod google;
pub mod images;
pub mod qr;
pub mod subject;

// pub use apple::ApplePassBuilder;   // Restored in PLAN-05 (apple builder body lands)
// pub use config::{AppleConfig, GoogleConfig, WalletConfig};   // Restored in PLAN-03
pub use error::WalletError;
// pub use google::GoogleWalletBuilder;   // Restored in PLAN-07 (google builder body lands)
pub use subject::{
    auto_foreground, Branding, Field, FieldAlignment, GeoPoint, PassKind, RgbColor, TextColorMode,
    WalletSubject,
};
