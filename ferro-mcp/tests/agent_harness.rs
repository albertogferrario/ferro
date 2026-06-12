//! COMP-03 agent-success-rate harness.
//!
//! This test file measures whether an LLM agent — reading `ferro-mcp` introspection
//! tools — can produce a working projection (a `ServiceDef` that renders and passes
//! the projection checkpoint) from a natural-language description.
//!
//! ## Execution paths
//!
//! - **Replay path** (default `cargo test`): deserializes committed transcript
//!   `ServiceDef`s and runs them through the T1–T4 scorer. No LLM, no network,
//!   fully deterministic. This is what CI validates on every push.
//!
//! - **Live path** (`FERRO_AGENT_EVAL=1`): stands up an in-process rmcp client over
//!   the real `FerroMcpService` dev tools, drives `claude-opus-4-8` through a
//!   tool-use loop, captures final `ServiceDef` JSON into committed transcripts, and
//!   writes a baseline artifact. Gated behind `#[ignore]` + env-var check so normal
//!   `cargo test` / CI skips it entirely (no API key, no network, no LLM flakiness).
//!
//! ## Tier definitions (stated before any run — D-07)
//!
//! - **T1 Structural validity**: the `ServiceDef` deserializes AND
//!   `Spec::from_service_def` renders AND `Catalog::validate(&spec)` returns 0 errors.
//! - **T2 Intent coverage**: `derive_intents(&service)[0].intent` equals the task's
//!   declared target intent. Presence of `intent_hints` disqualifies T2 — the agent
//!   must derive intent structurally, not declare it.
//! - **T3 Functional completeness**: the rendered spec's primary content element is
//!   data-bound per the Phase 213 bar (Browse/Track `DataTable` with non-empty
//!   `columns` + `items_path`; Process `KanbanBoard` with `columns` + `items_path` +
//!   `group_by`; Collect `Form` with ≥1 field; Summarize `StatCard` with `value_path`;
//!   Focus/Analyze primary fields bound).
//! - **T4 Checkpoint pass**: `checkpoint_projection` returns a verdict with
//!   `status != Fail` (zero blocking findings).
//!
//! Tiers are cumulative: a trial passes T_n iff it passes T_1 through T_n.
//!
//! ## Wave structure
//!
//! - **Wave 1 (this file, Plan 01)**: corpus fixture + contamination guard.
//! - **Wave 2 (Plan 02)**: scorer struct (`TierResult`) + replay infrastructure.
//! - **Wave 3 (Plan 03)**: live agent loop + prompt template.
//! - **Wave 4 (Plan 04)**: baseline artifact + discovered-weaknesses section.

// Corpus is loaded at compile time by the contamination guard below.
// Later waves will also use it for the replay and live paths.

// ---------------------------------------------------------------------------
// Task 3: Contamination guard — always runs in default `cargo test`, no LLM.
// ---------------------------------------------------------------------------

/// Verifies the task corpus satisfies two invariants without any LLM or network call:
///
/// 1. The corpus contains exactly 14 tasks, 2 per intent across all 7 intents.
/// 2. No task description or id contains any word from the contamination denylist
///    (derived from `ferro-projections/tests/catalog.rs` + project memory).
///
/// This is the standing CI invariant for D-10 (contamination guard).
#[test]
fn corpus_contamination_guard() {
    // Load corpus at compile time so CI catches a missing fixture immediately.
    let raw = include_str!("fixtures/agent_harness/corpus.json");
    let corpus: Vec<serde_json::Value> =
        serde_json::from_str(raw).expect("corpus.json must be valid JSON");

    // Invariant 1: exactly 14 tasks.
    let n = corpus.len();
    assert_eq!(
        n,
        14,
        "corpus must have exactly 14 tasks (2 per intent × 7 intents); got {n}"
    );

    // Invariant 2: 2 tasks per intent across all 7 intents.
    use std::collections::HashMap;
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for task in &corpus {
        let intent = task["target_intent"]
            .as_str()
            .expect("each task must have a string target_intent");
        *counts.entry(intent).or_insert(0) += 1;
    }

    let expected_intents = [
        "Browse",
        "Focus",
        "Collect",
        "Process",
        "Summarize",
        "Analyze",
        "Track",
    ];
    for intent in &expected_intents {
        let count = counts.get(*intent).copied().unwrap_or(0);
        assert_eq!(
            count,
            2,
            "intent '{intent}' must appear exactly twice in the corpus; counts = {counts:?}"
        );
    }
    let distinct = counts.len();
    let keys: Vec<_> = counts.keys().collect();
    assert_eq!(
        distinct,
        7,
        "corpus must cover exactly 7 distinct intents; got {keys:?}"
    );

    // Invariant 3: contamination denylist — nouns sourced from
    // ferro-projections/tests/catalog.rs + project memory (gestiscilo, Italian state names).
    // These are the domains the corpus must NOT reference so the agent must derive
    // intent structurally rather than pattern-matching ferro's own examples.
    const DENYLIST: &[&str] = &[
        "product",
        "order",
        "invoice",
        "booking",
        "customer",
        "user",
        "shipment",
        "line_item",
        "payment",
        "warehouse",
        "category",
        "profile",
        "financials",
        "dashboard",
        "timeseries",
        "catalog",
        "registration",
        "secret",
        "auth",
        "staff",
        "gestiscilo",
        "confermato",
        "in_corso",
        "rientrato",
        "chiuso",
        "annullato",
        // Additional nouns from catalog.rs fixtures
        "article",
        "approval",
        "revenue",
        "sales",
        "variant",
        "publication",
    ];

    for task in &corpus {
        let id = task["id"].as_str().unwrap_or("?");
        let description = task["description"].as_str().unwrap_or("").to_lowercase();
        let id_lower = id.to_lowercase();

        for noun in DENYLIST {
            assert!(
                !description.contains(*noun),
                "corpus task '{id}' description contains denylist noun '{noun}'"
            );
            assert!(
                !id_lower.contains(*noun),
                "corpus task id '{id}' contains denylist noun '{noun}'"
            );
        }
    }
}
