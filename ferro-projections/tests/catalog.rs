//! Synthetic regression catalog for `derive_intents()` (COMP-02).
//!
//! Seven canonical `ServiceDef` fixtures (one per structural intent), structural-invariant
//! assertions, adversarial competing-signal fixtures, an `insta` snapshot per canonical shape,
//! and a `proptest` engine-robustness invariant. The system under test
//! (`ferro-projections/src/derive.rs`) is read-only.
//!
//! ## Discovered weaknesses
//!
//! **Analyze↔Summarize margin is structurally thin.** The `datetime_numeric_cooccurrence` signal
//! contributes a single flat 0.35 raw weight regardless of how many DateTime fields are present.
//! In the calibrated fixture (2 writable + 1 read_only DateTime, 1 read_only Money), Analyze wins
//! by only 0.1429 normalized (runner_up=0.8571). Adding a second Money or Percentage field flips
//! the winner to Summarize immediately because each numeric-aggregate field adds 0.30 raw weight
//! while the co-occurrence signal does not scale with DateTime count. The calibrated ANALYZE_MARGIN
//! is 0.04 — the tightest margin in the catalog. A future `derive.rs` change that raises the
//! Summarize Money weight even slightly (e.g. 0.31/field) would flip this fixture without any
//! structural signal overlap being added. This is a genuine derivation limitation: the engine has
//! no way to distinguish a "time-series with one KPI" from a "dashboard with one KPI and a date".

use ferro_projections::derive_intents;
use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldMeaning, GuardDef, Intent, IntentScore, ServiceDef,
    StateDef, StateMachine, Transition,
};
use proptest::prelude::*;

// ---------------------------------------------------------------------------
// Confidence floor + margin constants — calibrated from first observed run.
//
// Calibration protocol (D-07):
//   floor  = observed_primary - 0.15   (15pp cushion below observation)
//   margin = observed_gap    - 0.10    (10pp cushion below observed gap)
//
// Observed values (2026-06-12):
//   browse:    primary=1.0000  runner_up=0.2414  gap=0.7586
//   focus:     primary=1.0000  runner_up=0.2593  gap=0.7407
//   collect:   primary=1.0000  runner_up=0.4000  gap=0.6000
//   process:   primary=1.0000  runner_up=0.1842  gap=0.8158
//   summarize: primary=1.0000  runner_up=0.1000  gap=0.9000
//   analyze:   primary=1.0000  runner_up=0.8571  gap=0.1429  ← thin margin (see doc weakness note)
//   track:     primary=1.0000  runner_up=0.4667  gap=0.5333
// ---------------------------------------------------------------------------

const BROWSE_FLOOR: f64 = 0.85; // 1.00 - 0.15
const BROWSE_MARGIN: f64 = 0.66; // 0.7586 - 0.10

const FOCUS_FLOOR: f64 = 0.85; // 1.00 - 0.15
const FOCUS_MARGIN: f64 = 0.64; // 0.7407 - 0.10

const COLLECT_FLOOR: f64 = 0.85; // 1.00 - 0.15
const COLLECT_MARGIN: f64 = 0.50; // 0.6000 - 0.10

const PROCESS_FLOOR: f64 = 0.85; // 1.00 - 0.15
const PROCESS_MARGIN: f64 = 0.72; // 0.8158 - 0.10

const SUMMARIZE_FLOOR: f64 = 0.85; // 1.00 - 0.15
const SUMMARIZE_MARGIN: f64 = 0.80; // 0.9000 - 0.10

const ANALYZE_FLOOR: f64 = 0.85; // 1.00 - 0.15
const ANALYZE_MARGIN: f64 = 0.04; // 0.1429 - 0.10  (thin — see Discovered weaknesses)

const TRACK_FLOOR: f64 = 0.85; // 1.00 - 0.15
const TRACK_MARGIN: f64 = 0.43; // 0.5333 - 0.10

// ---------------------------------------------------------------------------
// Canonical ServiceDef fixtures — ground truth for Phase 210's agent harness.
// ---------------------------------------------------------------------------

pub mod fixtures {
    use super::*;

    /// Browse: entity navigation with collection structure.
    /// Two EntityName + two Category fields + two has_many relationships + simple CRUD.
    pub fn browse_catalog() -> ServiceDef {
        ServiceDef::new("product_catalog")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("sku", DataType::String, FieldMeaning::EntityName)
            .field("category", DataType::String, FieldMeaning::Category)
            .field("subcategory", DataType::String, FieldMeaning::Category)
            .has_many("variants", "product_variant")
            .has_many("images", "product_image")
            .action(ActionDef::new("create"))
            .action(ActionDef::new("update"))
            .action(ActionDef::new("delete"))
    }

    /// Focus: rich single-entity display with FreeText + ImageUrl + Url + inline/parent rels.
    pub fn focus_detail() -> ServiceDef {
        ServiceDef::new("article_detail")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("body", DataType::String, FieldMeaning::FreeText)
            .field("summary", DataType::String, FieldMeaning::FreeText)
            .field("cover_image", DataType::String, FieldMeaning::ImageUrl)
            .field("source_url", DataType::String, FieldMeaning::Url)
            .has_one("author", "user")
            .belongs_to("publication", "publication")
    }

    /// Collect: high writable ratio + write_only credential fields.
    pub fn collect_form() -> ServiceDef {
        ServiceDef::new("registration_form")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("name", DataType::String, FieldMeaning::EntityName)
            .field("email", DataType::String, FieldMeaning::Email)
            .field("phone", DataType::String, FieldMeaning::Phone)
            .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
            .write_only_field(
                "password_confirm",
                DataType::String,
                FieldMeaning::Sensitive,
            )
            .field("terms_accepted", DataType::Boolean, FieldMeaning::Boolean)
    }

    /// Process: guarded branching state machine + transition-trigger actions.
    pub fn process_workflow() -> ServiceDef {
        ServiceDef::new("approval_workflow")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("title", DataType::String, FieldMeaning::EntityName)
            .field("status", DataType::String, FieldMeaning::Status)
            .field("amount", DataType::Float, FieldMeaning::Money)
            .guard(GuardDef::new("has_required_fields"))
            .guard(GuardDef::new("is_approver"))
            .guard(GuardDef::new("is_cancellable"))
            .state_machine(
                StateMachine::new("approval_lifecycle")
                    .initial("draft")
                    .state(StateDef::new("draft"))
                    .state(StateDef::new("submitted"))
                    .state(StateDef::new("approved"))
                    .state(StateDef::new("rejected").final_state())
                    .state(StateDef::new("cancelled").final_state())
                    .transition(
                        Transition::new("draft", "submit", "submitted")
                            .guard("has_required_fields"),
                    )
                    .transition(
                        Transition::new("submitted", "approve", "approved").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("submitted", "reject", "rejected").guard("is_approver"),
                    )
                    .transition(
                        Transition::new("draft", "cancel", "cancelled").guard("is_cancellable"),
                    )
                    .transition(
                        Transition::new("submitted", "cancel", "cancelled").guard("is_cancellable"),
                    ),
            )
            .action(
                ActionDef::new("submit")
                    .precondition("has_required_fields")
                    .transition_trigger("submit"),
            )
            .action(
                ActionDef::new("approve")
                    .precondition("is_approver")
                    .transition_trigger("approve"),
            )
            .action(
                ActionDef::new("reject")
                    .precondition("is_approver")
                    .transition_trigger("reject"),
            )
            .action(
                ActionDef::new("cancel")
                    .precondition("is_cancellable")
                    .transition_trigger("cancel"),
            )
    }

    /// Summarize: multiple read-only Money/Percentage/Quantity fields (numeric aggregation).
    pub fn summarize_dashboard() -> ServiceDef {
        ServiceDef::new("revenue_summary")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .read_only_field("total_revenue", DataType::Float, FieldMeaning::Money)
            .read_only_field("average_order", DataType::Float, FieldMeaning::Money)
            .read_only_field("gross_margin", DataType::Float, FieldMeaning::Percentage)
            .read_only_field("conversion_rate", DataType::Float, FieldMeaning::Percentage)
            .read_only_field("unit_count", DataType::Integer, FieldMeaning::Quantity)
            .read_only_field("return_rate", DataType::Float, FieldMeaning::Percentage)
    }

    /// Analyze: DateTime co-occurring with sparse numeric measure.
    ///
    /// Signal arithmetic:
    ///   - 2 writable DateTime domain fields + 1 read_only DateTime: has_datetime=true
    ///   - 1 read_only Money: has_numeric=true → datetime_numeric_cooccurrence fires (+0.35 Analyze)
    ///   - Summarize from Money: +0.30; non_writable_ratio=50% (not >70%) → no mostly_read_only boost
    ///   - writable_ratio=50% (not >50%) → no high_writable_ratio Collect signal
    ///   - Analyze (0.35) > Summarize (0.30) → Analyze wins
    pub fn analyze_timeseries() -> ServiceDef {
        ServiceDef::new("sales_timeseries")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime)
            .field("period_start", DataType::DateTime, FieldMeaning::DateTime)
            .read_only_field("period_end", DataType::DateTime, FieldMeaning::DateTime)
            .read_only_field("revenue", DataType::Float, FieldMeaning::Money)
    }

    /// Track: linear state machine (no guards) + Status field.
    pub fn track_timeline() -> ServiceDef {
        ServiceDef::new("shipment_tracking")
            .field("id", DataType::Integer, FieldMeaning::Identifier)
            .field(
                "tracking_number",
                DataType::String,
                FieldMeaning::EntityName,
            )
            .field("status", DataType::String, FieldMeaning::Status)
            .field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
            .state_machine(
                StateMachine::new("shipment_lifecycle")
                    .initial("created")
                    .state(StateDef::new("created"))
                    .state(StateDef::new("picked_up"))
                    .state(StateDef::new("in_transit"))
                    .state(StateDef::new("out_for_delivery"))
                    .state(StateDef::new("delivered").final_state())
                    .transition(Transition::new("created", "pick_up", "picked_up"))
                    .transition(Transition::new("picked_up", "depart", "in_transit"))
                    .transition(Transition::new(
                        "in_transit",
                        "dispatch",
                        "out_for_delivery",
                    ))
                    .transition(Transition::new("out_for_delivery", "deliver", "delivered")),
            )
    }
}

// ---------------------------------------------------------------------------
// Canonical tests — one per structural intent.
// ---------------------------------------------------------------------------

#[test]
fn canonical_browse() {
    let svc = fixtures::browse_catalog();
    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Browse, "Browse must be primary");

    assert!(
        scores[0].confidence >= BROWSE_FLOOR,
        "Browse confidence {c} below floor {BROWSE_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= BROWSE_MARGIN,
            "Browse margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: >=2 EntityName/Category domain fields
    let entity_fields = svc
        .fields
        .iter()
        .filter(|f| matches!(f.meaning, FieldMeaning::EntityName | FieldMeaning::Category))
        .count();
    assert!(
        entity_fields >= 2,
        "Browse fixture needs >=2 entity/category fields, found {entity_fields}"
    );

    // Structural invariant: >=1 collection (has_many) relationship
    let collection_rels = svc
        .relationships
        .iter()
        .filter(|r| {
            matches!(
                r.cardinality,
                Cardinality::OneToMany | Cardinality::ManyToMany
            )
        })
        .count();
    assert!(
        collection_rels >= 1,
        "Browse fixture needs >=1 collection relationship, found {collection_rels}"
    );
}

#[test]
fn canonical_focus() {
    let svc = fixtures::focus_detail();
    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Focus, "Focus must be primary");

    assert!(
        scores[0].confidence >= FOCUS_FLOOR,
        "Focus confidence {c} below floor {FOCUS_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= FOCUS_MARGIN,
            "Focus margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: >=2 FreeText/ImageUrl/Url domain fields
    let rich_fields = svc
        .fields
        .iter()
        .filter(|f| {
            matches!(
                f.meaning,
                FieldMeaning::FreeText | FieldMeaning::ImageUrl | FieldMeaning::Url
            )
        })
        .count();
    assert!(
        rich_fields >= 2,
        "Focus fixture needs >=2 FreeText/ImageUrl/Url fields, found {rich_fields}"
    );
}

#[test]
fn canonical_collect() {
    let svc = fixtures::collect_form();
    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Collect, "Collect must be primary");

    assert!(
        scores[0].confidence >= COLLECT_FLOOR,
        "Collect confidence {c} below floor {COLLECT_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= COLLECT_MARGIN,
            "Collect margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: >=4 non-system fields
    let non_system = svc
        .fields
        .iter()
        .filter(|f| {
            !matches!(
                f.meaning,
                FieldMeaning::Identifier
                    | FieldMeaning::CreatedAt
                    | FieldMeaning::UpdatedAt
                    | FieldMeaning::ForeignKey
            )
        })
        .count();
    assert!(
        non_system >= 4,
        "Collect fixture needs >=4 non-system fields, found {non_system}"
    );

    // Structural invariant: majority of non-system fields are writable
    let non_sys_fields: Vec<_> = svc
        .fields
        .iter()
        .filter(|f| {
            !matches!(
                f.meaning,
                FieldMeaning::Identifier
                    | FieldMeaning::CreatedAt
                    | FieldMeaning::UpdatedAt
                    | FieldMeaning::ForeignKey
            )
        })
        .collect();
    let writable = non_sys_fields.iter().filter(|f| f.writable).count();
    let total = non_sys_fields.len();
    assert!(
        writable > total / 2,
        "Collect fixture needs majority writable fields: {writable}/{total} writable"
    );
}

#[test]
fn canonical_process() {
    let svc = fixtures::process_workflow();

    // Validate fixture structural correctness before derivation
    assert!(
        svc.validate().is_ok(),
        "process fixture must be structurally valid: {:?}",
        svc.validate()
    );

    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Process, "Process must be primary");

    assert!(
        scores[0].confidence >= PROCESS_FLOOR,
        "Process confidence {c} below floor {PROCESS_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= PROCESS_MARGIN,
            "Process margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: state machine is Some
    assert!(
        svc.state_machine.is_some(),
        "Process fixture must have a state machine"
    );

    // Structural invariant: >=2 guarded transitions
    let guarded_count = svc
        .state_machine
        .as_ref()
        .unwrap()
        .transitions
        .iter()
        .filter(|t| t.guard.is_some())
        .count();
    assert!(
        guarded_count >= 2,
        "Process fixture needs >=2 guarded transitions, found {guarded_count}"
    );

    // Structural invariant: >=2 branching states (states with >1 outgoing transition)
    let sm = svc.state_machine.as_ref().unwrap();
    let mut outgoing: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for t in &sm.transitions {
        *outgoing.entry(t.from.as_str()).or_insert(0) += 1;
    }
    let branching = outgoing.values().filter(|&&c| c > 1).count();
    assert!(
        branching >= 2,
        "Process fixture needs >=2 branching states, found {branching}"
    );
}

#[test]
fn canonical_summarize() {
    let svc = fixtures::summarize_dashboard();
    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].intent,
        Intent::Summarize,
        "Summarize must be primary"
    );

    assert!(
        scores[0].confidence >= SUMMARIZE_FLOOR,
        "Summarize confidence {c} below floor {SUMMARIZE_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= SUMMARIZE_MARGIN,
            "Summarize margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: >=3 Money/Percentage/Quantity fields
    let numeric_fields = svc
        .fields
        .iter()
        .filter(|f| {
            matches!(
                f.meaning,
                FieldMeaning::Money | FieldMeaning::Percentage | FieldMeaning::Quantity
            )
        })
        .count();
    assert!(
        numeric_fields >= 3,
        "Summarize fixture needs >=3 numeric (Money/Percentage/Quantity) fields, found {numeric_fields}"
    );

    // Structural invariant: all domain (non-system) fields are non-writable
    let domain_writable = svc
        .fields
        .iter()
        .filter(|f| {
            !matches!(
                f.meaning,
                FieldMeaning::Identifier
                    | FieldMeaning::CreatedAt
                    | FieldMeaning::UpdatedAt
                    | FieldMeaning::ForeignKey
            )
        })
        .filter(|f| f.writable)
        .count();
    assert!(
        domain_writable == 0,
        "Summarize fixture must have all domain fields non-writable, found {domain_writable} writable"
    );
}

#[test]
fn canonical_analyze() {
    let svc = fixtures::analyze_timeseries();
    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Analyze, "Analyze must be primary");

    assert!(
        scores[0].confidence >= ANALYZE_FLOOR,
        "Analyze confidence {c} below floor {ANALYZE_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= ANALYZE_MARGIN,
            "Analyze margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: >=1 DateTime domain field
    let datetime_fields = svc
        .fields
        .iter()
        .filter(|f| matches!(f.meaning, FieldMeaning::DateTime))
        .count();
    assert!(
        datetime_fields >= 1,
        "Analyze fixture needs >=1 DateTime field, found {datetime_fields}"
    );

    // Structural invariant: primary score emits datetime_numeric_cooccurrence signal
    assert!(
        scores[0]
            .matching_signals
            .iter()
            .any(|s| s.contains("datetime_numeric_cooccurrence")),
        "Analyze primary score must contain datetime_numeric_cooccurrence signal; got: {:?}",
        scores[0].matching_signals
    );
}

#[test]
fn canonical_track() {
    let svc = fixtures::track_timeline();

    // Validate fixture structural correctness before derivation
    assert!(
        svc.validate().is_ok(),
        "track fixture must be structurally valid: {:?}",
        svc.validate()
    );

    let scores = derive_intents(&svc);

    assert!(!scores.is_empty());
    assert_eq!(scores[0].intent, Intent::Track, "Track must be primary");

    assert!(
        scores[0].confidence >= TRACK_FLOOR,
        "Track confidence {c} below floor {TRACK_FLOOR}",
        c = scores[0].confidence,
    );
    if scores.len() > 1 {
        assert!(
            scores[0].confidence - scores[1].confidence >= TRACK_MARGIN,
            "Track margin too narrow: {p} vs {r}",
            p = scores[0].confidence,
            r = scores[1].confidence,
        );
    }

    // Structural invariant: state machine is Some
    assert!(
        svc.state_machine.is_some(),
        "Track fixture must have a state machine"
    );

    // Structural invariant: no transition has a guard (linear, unguarded progression)
    let guarded_transitions = svc
        .state_machine
        .as_ref()
        .unwrap()
        .transitions
        .iter()
        .filter(|t| t.guard.is_some())
        .count();
    assert!(
        guarded_transitions == 0,
        "Track fixture must have no guarded transitions, found {guarded_transitions}"
    );

    // Structural invariant: >=3 non-final states
    let non_final = svc
        .state_machine
        .as_ref()
        .unwrap()
        .states
        .iter()
        .filter(|s| !s.is_final)
        .count();
    assert!(
        non_final >= 3,
        "Track fixture needs >=3 non-final states, found {non_final}"
    );

    // Structural invariant: primary score contains linear_states signal
    assert!(
        scores[0]
            .matching_signals
            .iter()
            .any(|s| s.contains("linear_states")),
        "Track primary score must contain linear_states signal; got: {:?}",
        scores[0].matching_signals
    );
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

/// Redacted view of an IntentScore for snapshots: signals only, no confidence floats.
/// Per D-04: confidence values must not appear in snapshot payload.
#[derive(serde::Serialize)]
struct IntentSignals<'a> {
    intent: String,
    signals: &'a [String],
}

fn redacted_signals(scores: &[IntentScore]) -> Vec<IntentSignals<'_>> {
    scores
        .iter()
        .map(|s| IntentSignals {
            intent: format!("{:?}", s.intent),
            signals: &s.matching_signals,
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Snapshot tests — one per canonical intent (D-03/D-04).
// Snapshots capture ranked (intent, signals) only; no confidence floats.
// ---------------------------------------------------------------------------

#[test]
fn snapshot_canonical_browse() {
    let svc = fixtures::browse_catalog();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_browse", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_focus() {
    let svc = fixtures::focus_detail();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_focus", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_collect() {
    let svc = fixtures::collect_form();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_collect", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_process() {
    let svc = fixtures::process_workflow();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_process", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_summarize() {
    let svc = fixtures::summarize_dashboard();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_summarize", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_analyze() {
    let svc = fixtures::analyze_timeseries();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_analyze", redacted_signals(&scores));
}

#[test]
fn snapshot_canonical_track() {
    let svc = fixtures::track_timeline();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_track", redacted_signals(&scores));
}

// ---------------------------------------------------------------------------
// Adversarial tests — confusable intent pairs (D-06).
// Each fixture resolves a genuine signal competition; winner annotated inline.
// ---------------------------------------------------------------------------

#[test]
fn adversarial_browse_vs_summarize() {
    // competing: entity_name+category (Browse baseline ~0.50) vs money_fields (Summarize 0.90);
    // Summarize must win because per-field money signal (0.3×3=0.90) outweighs entity_name
    // accumulation (0.2×1 + 0.1×2 = 0.4) plus Browse baseline (0.1) = 0.50 raw Browse
    let svc = ServiceDef::new("product_pricing")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("category", DataType::String, FieldMeaning::Category)
        .field("subcategory", DataType::String, FieldMeaning::Category)
        .read_only_field("list_price", DataType::Float, FieldMeaning::Money)
        .read_only_field("sale_price", DataType::Float, FieldMeaning::Money)
        .read_only_field("cost", DataType::Float, FieldMeaning::Money);
    let scores = derive_intents(&svc);
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].intent,
        Intent::Summarize,
        "Summarize must beat Browse despite entity_name+category fields; got {:?}",
        scores[0].intent
    );
}

#[test]
fn adversarial_process_vs_track() {
    // competing: guarded_transitions (Process) vs linear_states+unguarded (Track);
    // Process must win because branching factor + guard density dominates Track's linear signal
    let svc = ServiceDef::new("mixed_workflow")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("title", DataType::String, FieldMeaning::EntityName)
        .field("status", DataType::String, FieldMeaning::Status)
        .field("amount", DataType::Float, FieldMeaning::Money)
        .guard(GuardDef::new("is_authorized"))
        .guard(GuardDef::new("is_complete"))
        .guard(GuardDef::new("is_cancellable"))
        .state_machine(
            StateMachine::new("mixed_lifecycle")
                .initial("draft")
                .state(StateDef::new("draft"))
                .state(StateDef::new("pending"))
                .state(StateDef::new("approved"))
                .state(StateDef::new("rejected").final_state())
                .transition(Transition::new("draft", "submit", "pending").guard("is_complete"))
                .transition(
                    Transition::new("pending", "approve", "approved").guard("is_authorized"),
                )
                .transition(Transition::new("pending", "reject", "rejected").guard("is_authorized"))
                .transition(Transition::new("draft", "cancel", "rejected").guard("is_cancellable")),
        )
        .action(
            ActionDef::new("submit")
                .precondition("is_complete")
                .transition_trigger("submit"),
        )
        .action(
            ActionDef::new("approve")
                .precondition("is_authorized")
                .transition_trigger("approve"),
        )
        .action(
            ActionDef::new("reject")
                .precondition("is_authorized")
                .transition_trigger("reject"),
        );
    let scores = derive_intents(&svc);
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].intent,
        Intent::Process,
        "Process must beat Track despite Status field; got {:?}",
        scores[0].intent
    );
}

#[test]
fn adversarial_analyze_vs_summarize() {
    // competing: datetime_numeric_cooccurrence (Analyze 0.35) vs money_fields (Summarize 0.30);
    // Analyze must win because temporal density outweighs single monetary measure in time-series
    // context (writable_ratio=50% prevents mostly_read_only Summarize boost)
    let svc = ServiceDef::new("kpi_timeseries")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("measured_at", DataType::DateTime, FieldMeaning::DateTime)
        .field("window_start", DataType::DateTime, FieldMeaning::DateTime)
        .read_only_field("window_end", DataType::DateTime, FieldMeaning::DateTime)
        .read_only_field("kpi_value", DataType::Float, FieldMeaning::Money);
    let scores = derive_intents(&svc);
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].intent,
        Intent::Analyze,
        "Analyze must beat Summarize with sparse money and dense datetime; got {:?}",
        scores[0].intent
    );
}

#[test]
fn adversarial_collect_vs_focus() {
    // competing: free_text+image_url (Focus ~0.60) vs high_writable_ratio+write_only (Collect 0.75);
    // Collect must win because write-only credential fields accumulate 0.4 on top of writable
    // ratio signal (0.35), totalling 0.75 raw vs Focus 0.50 field signals + 0.10 baseline = 0.60
    let svc = ServiceDef::new("profile_setup")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("bio", DataType::String, FieldMeaning::FreeText)
        .field("avatar_url", DataType::String, FieldMeaning::ImageUrl)
        .field("display_name", DataType::String, FieldMeaning::EntityName)
        .field("email", DataType::String, FieldMeaning::Email)
        .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
        .write_only_field(
            "password_confirm",
            DataType::String,
            FieldMeaning::Sensitive,
        );
    let scores = derive_intents(&svc);
    assert!(!scores.is_empty());
    assert_eq!(
        scores[0].intent,
        Intent::Collect,
        "Collect must beat Focus despite FreeText+ImageUrl fields; got {:?}",
        scores[0].intent
    );
}

// ---------------------------------------------------------------------------
// Proptest engine-robustness invariants (D-05).
// Asserts derive_intents() is total, bounded, sorted, and duplicate-free
// over arbitrary ServiceDef inputs.
// ---------------------------------------------------------------------------

fn arb_field_meaning() -> impl Strategy<Value = FieldMeaning> {
    prop_oneof![
        Just(FieldMeaning::EntityName),
        Just(FieldMeaning::Money),
        Just(FieldMeaning::Percentage),
        Just(FieldMeaning::Quantity),
        Just(FieldMeaning::FreeText),
        Just(FieldMeaning::ImageUrl),
        Just(FieldMeaning::Status),
        Just(FieldMeaning::Category),
        Just(FieldMeaning::DateTime),
        Just(FieldMeaning::Identifier),
        Just(FieldMeaning::Email),
        Just(FieldMeaning::Boolean),
    ]
}

fn arb_data_type() -> impl Strategy<Value = DataType> {
    prop_oneof![
        Just(DataType::String),
        Just(DataType::Integer),
        Just(DataType::Float),
        Just(DataType::Boolean),
        Just(DataType::DateTime),
    ]
}

fn arb_service_def() -> impl Strategy<Value = ServiceDef> {
    proptest::collection::vec((arb_data_type(), arb_field_meaning()), 0..8usize).prop_map(
        |fields| {
            let mut svc = ServiceDef::new("proptest_subject");
            for (i, (dt, meaning)) in fields.into_iter().enumerate() {
                svc = svc.field(format!("f_{i}"), dt, meaning);
            }
            svc
        },
    )
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 256,
        .. ProptestConfig::default()
    })]

    #[test]
    fn engine_never_panics_returns_valid_scores(svc in arb_service_def()) {
        let scores = derive_intents(&svc);

        // Invariant 1: never empty
        prop_assert!(!scores.is_empty());

        // Invariant 2: all confidence values in [0.0, 1.0]
        for s in &scores {
            prop_assert!(s.confidence >= 0.0, "confidence below 0: {}", s.confidence);
            prop_assert!(s.confidence <= 1.0, "confidence above 1: {}", s.confidence);
        }

        // Invariant 3: sorted descending by confidence
        for i in 1..scores.len() {
            prop_assert!(
                scores[i - 1].confidence >= scores[i].confidence,
                "not sorted at [{i}]: {} < {}",
                scores[i - 1].confidence,
                scores[i].confidence
            );
        }

        // Invariant 4: no duplicate Intent in output
        let intents: Vec<_> = scores.iter().map(|s| format!("{:?}", s.intent)).collect();
        let unique_count = {
            let mut deduped = intents.clone();
            deduped.sort();
            deduped.dedup();
            deduped.len()
        };
        prop_assert_eq!(intents.len(), unique_count, "duplicate intent in output");
    }
}
