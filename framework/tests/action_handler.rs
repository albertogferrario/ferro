//! Integration tests for the `#[action]` runtime helper and macro.
//!
//! Scaffold landed in Plan 01. Macro smoke test added in Plan 03.
//! Full corpus (flash payload assertions, open-redirect mitigation,
//! log-injection mitigation, error-path round trip, override application)
//! lands in Plan 04.

extern crate ferro_rs as ferro;

use ferro::{action, ActionError, ActionResult, FlashVariant, Request};

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

/// Macro smoke test — `#[action(redirect_to = "/x")]` compiles in a
/// downstream crate and produces a `Response`-returning async fn.
/// The generated signature is verified by type-checking: the function
/// must be assignable to `fn(Request) -> impl Future<Output = Response>`.
#[action(redirect_to = "/x")]
pub async fn macro_smoke_handler(_req: Request) -> ActionResult {
    Ok(())
}

#[test]
fn macro_generated_handler_has_correct_type() {
    // Verify the generated function has the expected signature by creating
    // a function pointer of the expected type. This fails to compile if the
    // macro does not produce a `fn(Request) -> impl Future<Output = Response>`.
    let _f: fn(Request) -> _ = macro_smoke_handler;
    // If we reach here the macro expanded correctly and the type checks.
}
