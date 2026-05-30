//! Integration tests for the `#[action]` runtime helper.
//!
//! Scaffold landed in Plan 01. Full corpus (flash payload assertions,
//! open-redirect mitigation, log-injection mitigation, error-path round
//! trip, override application) lands in Plan 04.

extern crate ferro_rs as ferro;

use ferro::{ActionError, ActionResult, FlashVariant};

/// Smoke test — the public API surface compiles in a downstream crate.
/// Plan 04 replaces this with the full corpus.
#[test]
fn public_surface_compiles() {
    let _r: ActionResult = Ok(());
    let _e = ActionError::msg("smoke");
    let _e2 = ActionError::not_found("missing");
    let _e3 = ActionError::forbidden("nope");
    let _e4 = ActionError::unauthorized("login")
        .with_flash(FlashVariant::Warning)
        .redirect_to("/login");
}
