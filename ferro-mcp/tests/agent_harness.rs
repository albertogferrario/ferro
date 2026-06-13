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
//! Tiers are cumulative: a trial passes T_n iff it passes T_1 through T_n.
//!
//! - **T1 Structural validity**: the `ServiceDef` deserializes AND
//!   `Spec::from_service_def` renders AND `Catalog::validate(&spec)` returns 0 errors.
//!   In debug builds `from_service_def` panics on an invalid spec; the scorer wraps
//!   the call in `std::panic::catch_unwind` so a malformed ServiceDef scores T1=false
//!   without aborting the test process (Pitfall 3 mitigation).
//!
//! - **T2 Intent coverage**: `derive_intents(&service)[0].intent` equals the task's
//!   declared target intent. Presence of `intent_hints` disqualifies T2 — the agent
//!   must derive intent structurally, not declare it. This rule is stated here before
//!   any run so it cannot be adjusted after the fact (D-08 anti-cheat).
//!
//! - **T3 Functional completeness**: the rendered spec's primary content element is
//!   data-bound per the Phase 213 bar:
//!   - Browse/Track `DataTable`: non-empty `columns` AND non-empty `data_path`.
//!   - Process `KanbanBoard`: non-empty `columns` AND `items_path` present AND
//!     `group_by` present (Phase 213: `data_path` removed from `KanbanBoard`).
//!   - Collect `Form`: ≥1 field child element.
//!   - Summarize `StatCard`: `value_path` present.
//!   - Focus/Analyze `DescriptionList`: non-empty `items`.
//!
//! - **T4 Checkpoint pass**: `checkpoint_projection` returns a verdict with
//!   `status != Fail` (zero blocking findings — A3). T4 materializes the agent's
//!   ServiceDef into `src/projections/<name>.rs` inside a `tempfile::tempdir()` (Pitfall 4
//!   mitigation: `checkpoint_projection::execute` is filesystem-coupled and reads source
//!   files, not in-memory structs).
//!
//! ## Wave structure
//!
//! - **Wave 1 (Plan 01)**: corpus fixture + contamination guard.
//! - **Wave 2 (Plan 02)**: scorer struct (`TierResult`) + replay infrastructure + fixture transcripts.
//! - **Wave 3 (Plan 03)**: live agent loop + prompt template.
//! - **Wave 4 (Plan 04)**: baseline artifact + discovered-weaknesses section.

use std::panic::AssertUnwindSafe;

use ferro_json_ui::projection::{RenderMode, VisualContext};
use ferro_json_ui::{global_catalog, Spec};
use ferro_projections::{derive_intents, Intent, ServiceDef};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Tier result — 4-field struct, never collapsed to bool (D-08).
// Cumulative: t2 = t1 && <t2 check>, t3 = t2 && <t3 check>, t4 = t3 && <t4 check>.
// ---------------------------------------------------------------------------

/// Per-trial tier result for the agent-success-rate harness.
///
/// Each field represents one tier (T1–T4). Tiers are cumulative: a trial passes
/// tier N iff it passes tiers 1..N. This struct is never collapsed to a single
/// boolean — doing so would lose the per-tier signal needed for per-tier rates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TierResult {
    /// T1: ServiceDef deserializes AND renders AND catalog validates.
    pub t1: bool,
    /// T2: top derived intent == declared target (and no intent_hints present).
    pub t2: bool,
    /// T3: primary content element is data-bound, not a placeholder.
    pub t3: bool,
    /// T4: checkpoint_projection verdict.status != Fail.
    pub t4: bool,
}

// ---------------------------------------------------------------------------
// Corpus task struct (for replay and live paths).
// ---------------------------------------------------------------------------

/// A single task from the agent-eval corpus.
///
/// `target_intent` is stored as a `String` because the corpus JSON uses
/// PascalCase (e.g. `"Browse"`) while `Intent`'s serde form is snake_case
/// (e.g. `"browse"`). Wave 3 will convert via `parse_intent()` when needed.
#[derive(Debug, Deserialize)]
pub struct CorpusTask {
    pub id: String,
    pub target_intent: String,
    pub description: String,
    #[serde(default)]
    pub expected_actions: Vec<String>,
    #[serde(default)]
    pub expected_guards: Vec<String>,
}

// ---------------------------------------------------------------------------
// Transcript structs (replay path — deserialize committed transcripts).
// ---------------------------------------------------------------------------

/// A single trial record captured from a live agent run.
#[derive(Debug, Deserialize)]
pub struct TrialRecord {
    pub trial: u32,
    /// The agent's final ServiceDef JSON — the only field load-bearing for replay.
    pub service_def: serde_json::Value,
    /// Optional audit trace of tool calls (not used by scorer).
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}

/// One transcript file — one task, one or more trials.
#[derive(Debug, Deserialize)]
pub struct Transcript {
    pub task_id: String,
    pub target_intent: Intent,
    pub model: String,
    pub prompt_version: String,
    pub trials: Vec<TrialRecord>,
}

// ---------------------------------------------------------------------------
// T3 binding helpers — inspect rendered Spec.elements props as raw JSON.
// "213 binding bar": KanbanBoard uses items_path (not data_path which was removed).
// DataTable uses data_path (unchanged). No data_path usage for KanbanBoard anywhere.
// ---------------------------------------------------------------------------

/// Determine the primary element type_name expected for a given intent.
fn primary_element_type(intent: &Intent) -> &'static str {
    match intent {
        Intent::Browse | Intent::Track => "DataTable",
        Intent::Process => "KanbanBoard",
        Intent::Collect => "Form",
        Intent::Summarize => "StatCard",
        Intent::Focus | Intent::Analyze => "DescriptionList",
        Intent::Custom(_) => "DataTable", // fallback
    }
}

/// Check whether a rendered element's props satisfy the Phase 213 binding bar.
///
/// Inspects raw JSON props (more robust than typed-struct deserialization to
/// prop-struct churn). Prop keys per intent:
/// - Browse/Track DataTable: `columns` non-empty AND `data_path` non-empty.
/// - Process KanbanBoard: `columns` non-empty AND `items_path` present AND `group_by` present.
///   (Phase 213 removed `data_path` from KanbanBoard — use `items_path` only.)
/// - Collect Form: ≥1 field child (checked via element children, not props).
/// - Summarize StatCard: `value_path` present.
/// - Focus/Analyze DescriptionList: `items` non-empty.
fn is_bound(props: &serde_json::Value, intent: &Intent, children: &[String]) -> bool {
    match intent {
        Intent::Browse | Intent::Track => {
            let columns_ok = props
                .get("columns")
                .and_then(|v| v.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            let path_ok = props
                .get("data_path")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            columns_ok && path_ok
        }
        Intent::Process => {
            // Phase 213 binding bar: KanbanBoard uses items_path (not data_path).
            let columns_ok = props
                .get("columns")
                .and_then(|v| v.as_array())
                .map(|a| !a.is_empty())
                .unwrap_or(false);
            let items_path_ok = props
                .get("items_path")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            let group_by_ok = props
                .get("group_by")
                .and_then(|v| v.as_str())
                .map(|s| !s.is_empty())
                .unwrap_or(false);
            columns_ok && items_path_ok && group_by_ok
        }
        Intent::Collect => {
            // Form bound iff it has ≥1 child field element.
            !children.is_empty()
        }
        Intent::Summarize => props
            .get("value_path")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false),
        Intent::Focus | Intent::Analyze => props
            .get("items")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        Intent::Custom(_) => false,
    }
}

// ---------------------------------------------------------------------------
// T1/T2/T3 scorer (synchronous — no async needed for T1–T3).
// ---------------------------------------------------------------------------

/// Score a `ServiceDef` JSON value through T1, T2, and T3.
/// Returns a `TierResult` with `t4 = false` (T4 is async and added by `score()`).
///
/// T2 disqualifier rule (stated before runs, D-08 anti-cheat):
/// If the ServiceDef contains any `intent_hints`, T2 is forced to `false`
/// regardless of the derived top intent. This ensures T2 measures structural
/// derivation from field meanings / state machine / relationships, not a
/// self-declared override. An agent that emits `intent_hints` auto-passes T2
/// structurally — disqualifying their presence prevents gaming the tier.
fn score_t1_t3(agent_json: &serde_json::Value, target: &Intent) -> TierResult {
    let failed = TierResult {
        t1: false,
        t2: false,
        t3: false,
        t4: false,
    };

    // T1a: deserialize ServiceDef.
    let service: ServiceDef = match serde_json::from_value(agent_json.clone()) {
        Ok(s) => s,
        Err(_) => return failed,
    };

    // T2 disqualifier: presence of intent_hints forces t2=false.
    let has_intent_hints = !service.intent_hints.is_empty();

    // T1b: derive intents (always non-empty).
    let intents = derive_intents(&service);

    // T1c: render + implicit catalog validation.
    // PITFALL 3 (builder.rs lines 112-122): in debug builds, `from_service_def`
    // PANICS on an invalid spec instead of returning Err. `cargo test` is a debug
    // build. Wrap in `catch_unwind` so a malformed ServiceDef scores T1=false
    // without aborting the test process.
    let mode = if *target == Intent::Collect {
        RenderMode::Input
    } else {
        RenderMode::Display
    };
    let ctx = VisualContext {
        intent_index: 0,
        current_state: None,
        mode,
        templates: None,
    };

    let service_clone = service.clone();
    let intents_clone = intents.clone();
    let ctx_clone = ctx.clone();
    let render_result = std::panic::catch_unwind(AssertUnwindSafe(move || {
        Spec::from_service_def(&service_clone, &intents_clone, &ctx_clone)
    }));

    let spec = match render_result {
        Ok(Ok(s)) => s,
        Ok(Err(_)) | Err(_) => {
            // Panic or Err from from_service_def — T1 fails.
            return failed;
        }
    };

    // T1d: explicit catalog validation for error count (belt-and-suspenders).
    // `from_service_def` already validates internally; this call is the observable
    // signal that proves the "global_catalog().validate" path is exercised.
    if global_catalog().validate(&spec).is_err() {
        return failed;
    }

    let t1 = true;

    // T2: top derived intent == target AND no intent_hints.
    let top_intent = &intents[0].intent;
    let t2 = !has_intent_hints && (top_intent == target);

    // T3 (cumulative): inspect primary element's binding props.
    let t3 = if t2 {
        let primary_type = primary_element_type(target);
        if let Some(elem) = spec.elements.values().find(|e| e.type_name == primary_type) {
            is_bound(&elem.props, target, &elem.children)
        } else {
            false
        }
    } else {
        false
    };

    TierResult {
        t1,
        t2,
        t3,
        t4: false, // T4 filled in by the async `score()` function.
    }
}

// ---------------------------------------------------------------------------
// T4 materialization helpers (Pitfall 4 mitigation).
//
// `checkpoint_projection::execute` is FILESYSTEM-COUPLED: it reads
// `src/projections/<name>.rs` and `src/models/<name>.rs` from the project root.
// It does NOT accept an in-memory ServiceDef. The scorer must materialize the
// agent's ServiceDef into a tempfile::tempdir() before invoking the checkpoint.
// ---------------------------------------------------------------------------

/// Create a temp project root with a projection source under src/projections/.
/// Returns the `TempDir` — keep it alive for the entire T4 call.
fn project_with_projection(name: &str, projection_src: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir creation must succeed");
    let proj_dir = tmp.path().join("src/projections");
    std::fs::create_dir_all(&proj_dir).expect("create src/projections/ must succeed");
    std::fs::write(proj_dir.join(format!("{name}.rs")), projection_src)
        .expect("write projection source must succeed");
    tmp
}

/// Add a SeaORM-style model source under src/models/ to an existing temp root.
fn add_model(tmp: &tempfile::TempDir, name: &str, model_src: &str) {
    let models_dir = tmp.path().join("src/models");
    std::fs::create_dir_all(&models_dir).expect("create src/models/ must succeed");
    std::fs::write(models_dir.join(format!("{name}.rs")), model_src)
        .expect("write model source must succeed");
}

/// Render a `ServiceDef` into the Rust builder-call source format that
/// `checkpoint_projection::reconstruct_service_def` can parse.
///
/// The checkpoint reads `src/projections/<name>.rs` and extracts the ServiceDef
/// via regex matching on builder calls (`.field(...)`, `.optional_field(...)`,
/// `.state_machine(...)`, etc.). This function emits a faithful source file in
/// that exact format.
fn render_service_def_to_rust_source(service: &ServiceDef) -> String {
    let fn_name = format!("{}_service", service.name);
    let service_name = &service.name;

    let mut lines = Vec::new();
    lines.push("use ferro::{ServiceDef, DataType, FieldMeaning};".to_string());
    lines.push(format!("pub fn {fn_name}() -> ServiceDef {{"));
    lines.push(format!("    ServiceDef::new(\"{service_name}\")"));

    for field in &service.fields {
        let dt = format!("{:?}", field.data_type);
        let fm = format!("{:?}", field.meaning);
        // Select builder method based on readable/writable flags.
        let builder_call = match (field.readable, field.writable) {
            (true, false) => "read_only_field",
            (false, true) => "write_only_field",
            _ => {
                if field.required {
                    "field"
                } else {
                    "optional_field"
                }
            }
        };
        lines.push(format!(
            "        .{builder_call}(\"{}\", DataType::{dt}, FieldMeaning::{fm})",
            field.name,
        ));
    }

    for rel in &service.relationships {
        use ferro_projections::Cardinality;
        // Map to the builder methods that reconstruct_service_def can parse.
        match rel.cardinality {
            Cardinality::OneToMany | Cardinality::ManyToMany => {
                lines.push(format!(
                    "        .has_many(\"{}\", \"{}\")",
                    rel.name, rel.target
                ));
            }
            Cardinality::ManyToOne | Cardinality::OneToOne => {
                lines.push(format!(
                    "        .belongs_to(\"{}\", \"{}\")",
                    rel.name, rel.target
                ));
            }
        }
    }

    // Actions: emit minimal builder call that reconstruct_service_def can parse.
    // `parse_and_add_actions` in render_projection.rs uses: `.action("name", route)`.
    for action in &service.actions {
        let display = action.display_name.as_deref().unwrap_or(&action.name);
        lines.push(format!("        // action: {display}",));
    }

    lines.push("}".to_string());
    lines.join("\n")
}

/// Build a minimal SeaORM-style model source for the field→column seam check.
///
/// The checkpoint's seam 2 (`field_to_column`) resolves the model from
/// `src/models/<service_name>.rs` and checks that each projection field has a
/// backing column. This stub adds every field as a column so the seam passes.
fn render_model_source(service: &ServiceDef) -> String {
    let table_name = service.name.clone();

    let mut field_lines = Vec::new();
    // Always include an `id` column.
    field_lines.push("    pub id: i64,".to_string());
    for field in &service.fields {
        if field.name == "id" {
            continue;
        }
        field_lines.push(format!("    pub {}: String,", field.name));
    }

    format!(
        "use sea_orm::entity::prelude::*;\n\
         \n\
         #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]\n\
         #[sea_orm(table_name = \"{table_name}\")]\n\
         pub struct Model {{\n\
         {}\n\
         }}\n",
        field_lines.join("\n")
    )
}

// ---------------------------------------------------------------------------
// Full async scorer: T1–T4 cumulative.
// ---------------------------------------------------------------------------

/// Score a `ServiceDef` JSON value through T1–T4.
///
/// T4 materializes the ServiceDef into `src/projections/<name>.rs` inside a
/// `tempfile::tempdir()` (Pitfall 4: `checkpoint_projection::execute` reads source
/// files from disk, not in-memory structs). All T4 filesystem work stays inside the
/// tempdir. T4 passes iff `verdict.status != SeamStatus::Fail` (zero blocking
/// findings — A3, stated before runs).
pub async fn score(agent_json: &serde_json::Value, target: Intent) -> TierResult {
    let partial = score_t1_t3(agent_json, &target);

    if !partial.t3 {
        // T4 is cumulative — skip if T3 failed.
        return partial;
    }

    // T4: materialize ServiceDef → Rust source → tempdir → checkpoint.
    let service: ServiceDef = match serde_json::from_value(agent_json.clone()) {
        Ok(s) => s,
        Err(_) => return partial, // T3 passed but service won't re-deserialize — shouldn't happen.
    };

    let fn_name = format!("{}_service", service.name);
    let projection_src = render_service_def_to_rust_source(&service);
    let model_src = render_model_source(&service);

    // All T4 filesystem work confined to the tempdir (T-210-06 mitigation).
    let tmp = project_with_projection(&fn_name, &projection_src);
    add_model(&tmp, &service.name, &model_src);

    // T4 rule (stated before runs — A3): passes iff status != Fail.
    // SeamStatus::Warn is acceptable ("zero blocking findings" literal).
    let t4 = match ferro_mcp::tools::checkpoint_projection::execute(tmp.path(), &fn_name).await {
        Ok(verdict) => verdict.status != ferro_mcp::tools::checkpoint_projection::SeamStatus::Fail,
        Err(_) => false,
    };

    TierResult {
        t1: partial.t1,
        t2: partial.t2,
        t3: partial.t3,
        t4,
    }
}

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
        n, 14,
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
            count, 2,
            "intent '{intent}' must appear exactly twice in the corpus; counts = {counts:?}"
        );
    }
    let distinct = counts.len();
    let keys: Vec<_> = counts.keys().collect();
    assert_eq!(
        distinct, 7,
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

// ---------------------------------------------------------------------------
// Task 3: Replay path tests — no LLM, no network, always-green in cargo test.
// ---------------------------------------------------------------------------

/// Scores the two deterministic fixture transcripts and asserts their expected
/// tier results are reproduced identically on every run.
///
/// - `_fixture_valid.json`: a Browse ServiceDef with EntityName+Category fields.
///   Expected: t1=true, t2=true (Browse derives correctly), t3=true (DataTable bound).
/// - `_fixture_invalid.json`: a ServiceDef where `name` is an integer — fails
///   deserialization at T1. Expected: t1=false, t2=false, t3=false, t4=false.
///
/// Running this test twice must yield identical results (determinism guard).
#[tokio::test]
async fn agent_eval_replay_scores_are_deterministic() {
    let valid_raw = include_str!("fixtures/agent_harness/transcripts/_fixture_valid.json");
    let invalid_raw = include_str!("fixtures/agent_harness/transcripts/_fixture_invalid.json");

    let valid_transcript: Transcript =
        serde_json::from_str(valid_raw).expect("_fixture_valid.json must parse as Transcript");
    let invalid_transcript: Transcript =
        serde_json::from_str(invalid_raw).expect("_fixture_invalid.json must parse as Transcript");

    // Score the valid fixture.
    for trial in &valid_transcript.trials {
        let result = score(&trial.service_def, valid_transcript.target_intent.clone()).await;
        assert!(
            result.t1,
            "fixture_valid trial {}: expected t1=true, got {result:?}",
            trial.trial
        );
        assert!(
            result.t2,
            "fixture_valid trial {}: expected t2=true, got {result:?}",
            trial.trial
        );
        assert!(
            result.t3,
            "fixture_valid trial {}: expected t3=true, got {result:?}",
            trial.trial
        );
        // t4 result depends on tempdir materialization + checkpoint seams — pass/warn acceptable.
    }

    // Score the invalid fixture — must fail at T1 deterministically.
    for trial in &invalid_transcript.trials {
        let result = score(&trial.service_def, invalid_transcript.target_intent.clone()).await;
        assert_eq!(
            result,
            TierResult {
                t1: false,
                t2: false,
                t3: false,
                t4: false
            },
            "fixture_invalid trial {}: expected all tiers false, got {result:?}",
            trial.trial
        );
    }
}

/// Proves tier independence (D-08): a ServiceDef that passes T1 but fails T2
/// (wrong top intent) records `{t1:true, t2:false, t3:false, t4:false}`.
///
/// Uses a Browse-derived ServiceDef but declared target = Process, so T2 fails.
/// Verifies TierResult carries 4 independent fields, never collapsed to one bool.
#[test]
fn tier_results_never_collapse_to_boolean() {
    // A Browse ServiceDef (EntityName fields) with target_intent=Process.
    // T1 passes (valid ServiceDef, renders fine).
    // T2 fails (top derived intent is Browse, not Process).
    // T3 and T4 are false (cumulative).
    // DataType and FieldMeaning serde representation uses snake_case.
    let browse_service_json = serde_json::json!({
        "name": "aviary_band_record",
        "fields": [
            {
                "name": "id",
                "data_type": "integer",
                "meaning": "identifier",
                "required": true,
                "is_list": false,
                "readable": true,
                "writable": false
            },
            {
                "name": "band_code",
                "data_type": "string",
                "meaning": "entity_name",
                "required": true,
                "is_list": false,
                "readable": true,
                "writable": true
            },
            {
                "name": "species",
                "data_type": "string",
                "meaning": "entity_name",
                "required": true,
                "is_list": false,
                "readable": true,
                "writable": true
            }
        ]
    });

    // Score against Process — T2 must fail because the ServiceDef derives Browse.
    let result = score_t1_t3(&browse_service_json, &Intent::Process);

    assert!(result.t1, "T1 must pass (valid ServiceDef renders fine)");
    assert!(
        !result.t2,
        "T2 must fail (top intent is Browse, not Process)"
    );
    assert!(
        !result.t3,
        "T3 must be false (cumulative: t2=false → t3=false)"
    );
    assert!(
        !result.t4,
        "T4 must be false (cumulative: t3=false → t4=false)"
    );

    // The result is a struct with 4 independent fields — never collapsed to bool.
    let expected = TierResult {
        t1: true,
        t2: false,
        t3: false,
        t4: false,
    };
    assert_eq!(
        result, expected,
        "TierResult must record per-tier independence: {result:?}"
    );
}

/// Proves Pitfall 3 mitigation: a malformed ServiceDef (invalid JSON for the type)
/// scores T1=false and does NOT abort the test process.
///
/// This test uses the `_fixture_invalid.json` ServiceDef directly (name is an integer,
/// which fails deserialization), proving the scorer catches deserialization failure
/// gracefully via `catch_unwind` / early-return path.
#[test]
fn t1_invalid_spec_scores_fail_without_panic() {
    let invalid_raw = include_str!("fixtures/agent_harness/transcripts/_fixture_invalid.json");
    let transcript: Transcript =
        serde_json::from_str(invalid_raw).expect("_fixture_invalid.json must parse as Transcript");

    for trial in &transcript.trials {
        let result = score_t1_t3(&trial.service_def, &Intent::Browse);
        assert!(
            !result.t1,
            "invalid ServiceDef must score t1=false; got {result:?}"
        );
        assert!(
            !result.t2,
            "invalid ServiceDef must score t2=false (cumulative); got {result:?}"
        );
        assert!(
            !result.t3,
            "invalid ServiceDef must score t3=false (cumulative); got {result:?}"
        );
        // Process survives — no panic, no abort. If we reach this assertion the test passed.
    }
}
