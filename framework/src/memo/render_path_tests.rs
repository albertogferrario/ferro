// SC-3 render-path proof — honest single-fetch assertion across a multi-intent render.
//
// D-05 finding (259-RESEARCH Pattern 8): the `ServiceDef → JsonUiRenderer` render pass is
// SCHEMA-ONLY — `Spec::from_service_def` takes a `&ServiceDef` (static schema) and
// `&[IntentScore]` (derived from schema signals) and returns a `Spec` without performing
// any I/O or data fetch. Therefore the memoized fetch is NOT inside the renderer itself;
// it is the data-loader a real multi-intent handler calls once before passing its result
// to each render call. This harness constructs that pattern honestly:
//
//   1. A `#[ferro::memoize] async fn` loader that increments a `static FETCH_COUNTER`.
//   2. A single `MEMO_STORE.scope(...)` wrapping two intent renders (Browse + Summarize)
//      over the SAME key.
//   3. An assertion that `FETCH_COUNTER == 1` — N intents, one fetch.
//
// The render calls use the REAL `derive_intents` (ferro_projections) and the REAL
// `JsonUiRenderer` / `Spec::from_service_def` (ferro_json_ui). No stub renderer.
//
// 259-RESEARCH Pattern 8 / D-05 honesty rule — do NOT fabricate a fetch inside the
// renderer that does not exist.

use std::sync::atomic::{AtomicUsize, Ordering};

use ferro_json_ui::projection::{RenderMode, VisualContext};
use ferro_projections::render::BaseContext;
use ferro_projections::{derive_intents, DataType, FieldMeaning, Intent, ServiceDef};

use crate::memo::{memo_scope, with_memo_scope};
use crate::memoize;

// ── Fetch counter ────────────────────────────────────────────────────────────

/// Counts how many times the underlying data-loader ran within the test.
///
/// `static` so the counter is shared across the async tasks in the memo scope;
/// reset to 0 at the start of each test.
static FETCH_COUNTER: AtomicUsize = AtomicUsize::new(0);

// ── Memoized loader ──────────────────────────────────────────────────────────

/// Simulated data loader: represents the single per-request DB fetch a real
/// multi-intent handler would issue before rendering each intent view.
///
/// Returns a representative data payload (a Vec of string IDs). In production
/// this would be a DB query; here the body just increments `FETCH_COUNTER`.
#[memoize]
async fn load_model_set(key: u32) -> Vec<String> {
    FETCH_COUNTER.fetch_add(1, Ordering::SeqCst);
    // Representative data payload — not used in the schema-only render,
    // but models what a real caller would pass to the presentation layer.
    vec![format!("item-{key}-a"), format!("item-{key}-b")]
}

// ── Service fixture ──────────────────────────────────────────────────────────

/// A `ServiceDef` whose field signals derive both Browse and Summarize intents.
///
/// `FieldMeaning::Money` adds a Summarize signal (see derive.rs:164-168);
/// Browse has a baseline that ensures it is always present. Both intents are
/// therefore present in the scored list for this fixture.
fn product_service() -> ServiceDef {
    ServiceDef::new("product")
        .display_name("Product")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("price", DataType::Float, FieldMeaning::Money)
        .field("stock", DataType::Integer, FieldMeaning::Quantity)
}

// ── SC-3 proof ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn render_path_single_fetch() {
    // Reset the counter to 0 at the start — tests may run in any order.
    FETCH_COUNTER.store(0, Ordering::SeqCst);

    let service = product_service();

    // Step 1: derive intents from the schema (schema-only, no I/O).
    let intents = derive_intents(&service);
    assert!(
        !intents.is_empty(),
        "product fixture must derive at least one intent"
    );

    // Step 2: find Browse and Summarize in the scored list.
    let browse_idx = intents
        .iter()
        .position(|s| matches!(s.intent, Intent::Browse))
        .expect("Browse must be present (baseline signal)");

    let summarize_idx = intents
        .iter()
        .position(|s| matches!(s.intent, Intent::Summarize))
        .expect("Summarize must be present (Money/Quantity field signals)");

    // Step 3: enter a memo scope and render both intents, calling the memoized
    //         loader with THE SAME KEY before each render — this is the pattern
    //         a real multi-intent handler uses.
    let (browse_spec, summarize_spec) = with_memo_scope(memo_scope(), async {
        // ── Render Browse ───────────────────────────────────────────────────
        // Load the data for this key (first call — must run the body).
        let _data_for_browse = load_model_set(1).await;

        let browse_ctx = VisualContext {
            base: BaseContext {
                intent_index: browse_idx,
                ..Default::default()
            },
            mode: RenderMode::Display,
            templates: None,
        };
        let browse_spec = ferro_json_ui::Spec::from_service_def(&service, &intents, &browse_ctx)
            .expect("Browse render must succeed");

        // ── Render Summarize ────────────────────────────────────────────────
        // Same key — must hit the memo cache (FETCH_COUNTER stays at 1).
        let _data_for_summarize = load_model_set(1).await;

        let summarize_ctx = VisualContext {
            base: BaseContext {
                intent_index: summarize_idx,
                ..Default::default()
            },
            mode: RenderMode::Display,
            templates: None,
        };
        let summarize_spec =
            ferro_json_ui::Spec::from_service_def(&service, &intents, &summarize_ctx)
                .expect("Summarize render must succeed");

        (browse_spec, summarize_spec)
    })
    .await;

    // ── Assertions ────────────────────────────────────────────────────────────

    // Both renders produced a valid Spec (the real render path ran).
    assert_eq!(
        browse_spec.schema, "ferro-json-ui/v2",
        "Browse Spec must have the correct schema version"
    );
    assert!(
        browse_spec.elements.contains_key(&browse_spec.root),
        "Browse Spec must have a root element"
    );

    assert_eq!(
        summarize_spec.schema, "ferro-json-ui/v2",
        "Summarize Spec must have the correct schema version"
    );
    assert!(
        summarize_spec.elements.contains_key(&summarize_spec.root),
        "Summarize Spec must have a root element"
    );

    // SC-3: N intents over one key issued exactly ONE underlying fetch.
    //
    // Both Browse and Summarize calls invoked `load_model_set(1)` — the memoized
    // loader ran only once because the second call hit the memo store cache.
    assert_eq!(
        FETCH_COUNTER.load(Ordering::SeqCst),
        1,
        "N intents over one key must issue exactly one underlying fetch (SC-3)"
    );
}
