//! End-to-end integration tests for the async validation public API.
//!
//! Exercises the full proactive-uniqueness path: crate-root public API only
//! (`ferro_rs::{AsyncValidator, AsyncValidationError, unique, required, string}`),
//! in-memory SQLite DB singleton, redirect-back shape.
//!
//! All tests MUST be `#[serial]` because the `DB` singleton is process-global.

#[path = "async_rule_fixture.rs"]
mod fixture;

use ferro_rs::{required, string};
use ferro_rs::{rules, unique, AsyncValidationError, AsyncValidator};
use fixture::{init_test_db, seed_widget};
use serde_json::json;
use serial_test::serial;

/// SC1: duplicate value produces a field-level `Validation` error, not a
/// panic or an infra (500) error.
#[tokio::test]
#[serial]
async fn duplicate_value_is_validation_error() {
    init_test_db().await;
    seed_widget(1, "taken").await;

    let data = json!({"slug": "taken"});
    let result = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;

    match result {
        Err(AsyncValidationError::Validation(ref e)) => {
            assert!(
                e.has("slug"),
                "expected a field error for 'slug', got: {e:?}"
            );
        }
        Err(AsyncValidationError::Infra(ref fe)) => {
            panic!("expected Validation error, got Infra: {fe}");
        }
        Ok(()) => panic!("expected Err(Validation), got Ok(())"),
    }
}

/// SC2: a free (non-duplicate) value passes.
#[tokio::test]
#[serial]
async fn free_value_passes() {
    init_test_db().await;
    seed_widget(1, "taken").await;

    let data = json!({"slug": "free"});
    let result = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;

    assert!(result.is_ok(), "expected Ok(()), got: {result:?}");
}

/// SC2 (exclude-self): an edit with `.ignore(id)` allows the record to keep
/// its own current slug.
#[tokio::test]
#[serial]
async fn exclude_self_passes_on_edit() {
    init_test_db().await;
    seed_widget(1, "taken").await;

    let data = json!({"slug": "taken"});
    let result = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug").ignore(1_i64))
        .validate_async()
        .await;

    assert!(
        result.is_ok(),
        "expected Ok(()) when excluding own row, got: {result:?}"
    );
}

/// SC3: sync failure (required) short-circuits before the async rule runs.
/// The error must be for "slug" with the sync message, not overwritten by `unique`.
#[tokio::test]
#[serial]
async fn sync_failure_skips_async() {
    init_test_db().await;

    // Empty value fails `required()` — `unique` must not run.
    let data = json!({"slug": ""});
    let result = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;

    match result {
        Err(AsyncValidationError::Validation(ref e)) => {
            assert!(e.has("slug"), "expected 'slug' error from sync rule");
            // The error message must come from `required`, not from `unique`.
            let msg = e.first("slug").expect("first error on slug");
            assert!(
                !msg.contains("taken"),
                "unique message must not appear when sync fails: {msg}"
            );
        }
        other => panic!("expected Err(Validation), got: {other:?}"),
    }
}

/// VALID-03 / redirect-back shape: the `Validation(e)` result from test 1
/// converts to an `ActionError` via `with_old_input` + `into_action_error`.
/// Does not spin up an HTTP server — verifies the type-level shape only.
#[tokio::test]
#[serial]
async fn redirect_back_shape() {
    init_test_db().await;
    seed_widget(2, "occupied").await;

    let data = json!({"slug": "occupied"});
    let result = AsyncValidator::new(&data)
        .rules("slug", rules![required(), string()])
        .async_rule("slug", unique("widgets", "slug"))
        .validate_async()
        .await;

    // Extract the ValidationError from the async result.
    let ve = match result {
        Err(AsyncValidationError::Validation(e)) => e,
        other => panic!("expected Err(Validation), got: {other:?}"),
    };

    // Chain the redirect-back shape exactly as a handler would.
    // `into_action_error` produces an ActionError (no HTTP server needed).
    let action_err = ve
        .with_old_input(&data)
        .into_action_error("/widgets/create");

    // The ActionError must be the validation-failed kind, not a 500.
    // We verify by confirming the conversion compiles and produces a
    // non-infra ActionError (ActionError::is_server_error() is not public,
    // but validation_failed always uses a redirect kind — verify via Debug).
    let debug_str = format!("{action_err:?}");
    assert!(
        !debug_str.contains("Internal"),
        "expected validation redirect ActionError, got: {debug_str}"
    );
}
