# Phase 207: COMP-02 Synthetic Regression Catalog - Pattern Map

**Mapped:** 2026-06-12
**Files analyzed:** 2 (one new test file, one manifest edit)
**Analogs found:** 2 / 2

---

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|---|---|---|---|---|
| `ferro-projections/tests/catalog.rs` | test (integration) | request-response (pure fn call → assert) | `ferro-projections/tests/generate_schemas.rs` + `ferro-projections/src/derive.rs` `#[cfg(test)]` + `ferro-projection/tests/proptest_properties.rs` | exact (import surface) + role-match (proptest idiom) |
| `ferro-projections/Cargo.toml` | manifest | — | `ferro-reservation/Cargo.toml` + `ferro-projection/Cargo.toml` | exact (proptest entry format + workspace-version conventions) |

---

## Pattern Assignments

### `ferro-projections/tests/catalog.rs` (integration test, pure-function assertions)

#### Analog 1 (import surface): `ferro-projections/tests/generate_schemas.rs`

**Imports pattern** (lines 4–8):
```rust
use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldDef, FieldMeaning, GuardDef, InputDef, Intent,
    IntentHint, IntentScore, NavigationHint, RelationshipDef, ServiceDef,
    StateDef, StateMachine, Transition, Warning,
};
```
`derive_intents` is not re-exported in that file but is pub in the crate. Add it explicitly:
```rust
use ferro_projections::derive_intents;
```

**Test layout** (lines 76–136): single `#[test]` function, no `#[ignore]`, no async, no DB setup. Pure function call followed by `assert!`/`assert_eq!`. This is the exact layout to copy for all canonical and adversarial tests.

---

#### Analog 2 (assertion style): `ferro-projections/src/derive.rs` `#[cfg(test)]` lines 588–1495+

**Helper pattern** (lines 596–601):
```rust
fn find_intent<'a>(scores: &'a [IntentScore], intent: &Intent) -> Option<&'a IntentScore> {
    scores.iter().find(|s| &s.intent == intent)
}

fn has_signal(score: &IntentScore, signal: &str) -> bool {
    score.matching_signals.iter().any(|s| s.contains(signal))
}
```
Copy both helpers into `catalog.rs` (or inline them — the module is a single file).

**Primary-identity assertion style** (lines 972–983):
```rust
let scores = derive_intents(&service);
assert!(!scores.is_empty(), "Must return at least one score");
assert_eq!(scores[0].intent, Intent::Browse, "Browse must be primary");
```

**Confidence-range assertion style** (lines 1022–1029):
```rust
for s in &scores {
    assert!(
        s.confidence >= 0.0 && s.confidence <= 1.0,
        "Confidence {} out of range for {:?}",
        s.confidence,
        s.intent
    );
}
```

**Sorted-descending assertion style** (lines 1031–1036):
```rust
for i in 1..scores.len() {
    assert!(
        scores[i - 1].confidence >= scores[i].confidence,
        "Scores must be sorted descending"
    );
}
```

**State-machine builder style** (lines 1107–1134):
```rust
ServiceDef::new("order")
    .state_machine(
        StateMachine::new("order_lifecycle")
            .initial("draft")
            .state(StateDef::new("draft"))
            .state(StateDef::new("completed").final_state())
            .transition(
                Transition::new("draft", "submit", "pending").guard("has_required_fields"),
            ),
    )
    .action(
        ActionDef::new("submit")
            .transition_trigger("submit")
            .precondition("has_required_fields"),
    )
```
Note: the unit test uses `crate::state::{StateDef, StateMachine, Transition}` internal imports. In `catalog.rs` (integration test, outside the crate) use `ferro_projections::{StateDef, StateMachine, Transition}`.

**Signal-name string matching** (lines 1238–1244):
```rust
let browse: Vec<_> = signals
    .iter()
    .filter(|s| s.0 == Intent::Browse && s.2.contains(SIGNAL_COLLECTION_RELATIONSHIPS))
    .collect();
```
`catalog.rs` does not have access to the private `SIGNAL_*` constants. Use string literals that match the signal names (verified from derive.rs lines 9–47):
- `"entity_name"`, `"category_field"`, `"collection_relationships"`, `"linear_states"`,
  `"guarded_transitions"`, `"datetime_numeric_cooccurrence"`, `"high_writable_ratio"`,
  `"write_only_fields"`, `"money_fields"`, `"percentage_fields"`, `"quantity_fields"`,
  `"baseline"`, `"status_field"`.

Matching pattern in integration test:
```rust
assert!(
    scores[0].matching_signals.iter().any(|s| s.contains("entity_name")),
    "Browse fixture must emit entity_name signal"
);
```

---

#### Analog 3 (proptest idiom): `ferro-projection/tests/proptest_properties.rs`

**File-level imports** (lines 12–13):
```rust
use proptest::prelude::*;
```

**ProptestConfig + proptest! block structure** (lines 92–115):
```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 32,
        .. ProptestConfig::default()
    })]

    #[test]
    fn proptest_apply_determinism(deltas in proptest::collection::vec(-50i32..50i32, 0..50)) {
        // ...assertions using prop_assert! and prop_assert_eq!
        prop_assert!(!scores.is_empty());
        prop_assert_eq!(a, b, "message {}", ctx);
    }
}
```
For `catalog.rs` the proptest cases should use 256 (engine is synchronous and fast; no async/DB overhead unlike these two analogs which use `block_on`). Use the same `ProptestConfig { cases: 256, .. ProptestConfig::default() }` shape.

**`prop_oneof!` + `Just(...)` Strategy pattern** (ferro-reservation lines 189–197):
```rust
fn arb_op() -> impl Strategy<Value = Op> {
    prop_oneof![
        Just(Op::Hold),
        Just(Op::Commit),
        Just(Op::Release(ReleaseReason::UserCancelled)),
    ]
}
```
Apply directly to `arb_field_meaning()` and `arb_data_type()` strategies for `catalog.rs`.

**`proptest::collection::vec(strategy, range)` for bounded collections** (ferro-projection line 99):
```rust
deltas in proptest::collection::vec(-50i32..50i32, 0..50)
```
In `catalog.rs` this becomes:
```rust
fields in proptest::collection::vec((arb_data_type(), arb_field_meaning()), 0..8usize)
```

**`prop_map` to build the subject from generated data** (not shown verbatim in analogs but standard proptest pattern consistent with workspace idiom):
```rust
.prop_map(|fields| {
    let mut svc = ServiceDef::new("proptest_subject");
    for (i, (dt, meaning)) in fields.into_iter().enumerate() {
        svc = svc.field(format!("f_{i}"), dt, meaning);
    }
    svc
})
```
Note: `svc.fields.len()` cannot be used mid-chain since `.field()` consumes `svc`. Use the enumerated index `i` from `into_iter().enumerate()` instead.

---

### `ferro-projections/Cargo.toml` (manifest, `[dev-dependencies]` section)

#### Analog: `ferro-reservation/Cargo.toml` (lines 37–40) and `ferro-projection/Cargo.toml` (lines 29–31)

**ferro-reservation [dev-dependencies]** (lines 37–40):
```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "sqlx-postgres", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
```

**ferro-projection [dev-dependencies]** (lines 29–31):
```toml
[dev-dependencies]
tokio = { version = "1", features = ["full"] }
sea-orm = { version = "1.0", features = ["sqlx-sqlite", "runtime-tokio-native-tls", "macros"] }
proptest = "1"
```

**Proptest entry format to copy** (identical in both): `proptest = "1"` (bare version string, no features, no path).

**New section for `ferro-projections/Cargo.toml`** (append after `[dependencies]`):
```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
proptest = "1"
```
`ferro-projections` is synchronous and has no async or DB deps — `tokio` and `sea-orm` are not needed. `insta` is new to the workspace. The `yaml` feature is required for `assert_yaml_snapshot!`.

---

## Shared Patterns

### Pattern: `serde::Serialize` for snapshot redaction struct
**Apply to:** `catalog.rs` snapshot tests (D-04)

The `insta::assert_yaml_snapshot!` requires the snapshot value to implement `serde::Serialize`. `ferro-projections` already has `serde` in `[dependencies]`. The redacted struct in `catalog.rs` needs the derive:

```rust
#[derive(serde::Serialize)]
struct IntentSignals<'a> {
    intent: String,
    signals: &'a [String],
}

fn redacted_signals(scores: &[IntentScore]) -> Vec<IntentSignals<'_>> {
    scores.iter().map(|s| IntentSignals {
        intent: format!("{:?}", s.intent),
        signals: &s.matching_signals,
    }).collect()
}
```
`IntentScore` derives `Debug` (verified: `intent.rs`), so `format!("{:?}", s.intent)` produces stable intent names like `"Browse"`, `"Focus"`, etc.

### Pattern: Confidence floor + margin constants
**Apply to:** all 7 canonical `#[test]` functions

The pattern from derive.rs unit tests uses named constants for expected values. Do the same in `catalog.rs`:

```rust
// Filled in after calibration run (D-07). Placeholder values shown;
// replace with observed_primary - 0.15 (floor) and
// observed_primary - observed_runner_up - 0.10 (margin).
const BROWSE_FLOOR: f64 = 0.0;   // calibrate after first run
const BROWSE_MARGIN: f64 = 0.0;  // calibrate after first run
// ... one pair per intent
```
Each canonical test then:
```rust
assert!(scores[0].confidence >= BROWSE_FLOOR,
    "Browse confidence {} below floor {}", scores[0].confidence, BROWSE_FLOOR);
if scores.len() > 1 {
    assert!(scores[0].confidence - scores[1].confidence >= BROWSE_MARGIN,
        "margin too narrow: {} vs {}", scores[0].confidence, scores[1].confidence);
}
```

### Pattern: `validate()` correctness assertion for Process/Track fixtures
**Source:** Pitfall 1 in RESEARCH.md; `service.rs` `validate()` method (public)

```rust
assert!(svc.validate().is_ok(),
    "fixture must be structurally valid: {:?}", svc.validate());
```
Assert this before the derivation assertions in every canonical test that uses a state machine.

### Pattern: Adversarial inline comment format
**Apply to:** all 4 adversarial test functions (D-06)

```rust
// competing: entity_name+category (Browse) vs money_fields (Summarize);
// Summarize must win because per-field money signal outweighs entity_name accumulation
```
Place immediately above the fixture builder call, not above the test function.

---

## No Analog Found

No files in this phase lack an analog. Both deliverables have direct workspace precedents.

---

## Metadata

**Analog search scope:** `ferro-projections/tests/`, `ferro-projections/src/derive.rs`, `ferro-projection/tests/`, `ferro-reservation/tests/`, `ferro-reservation/Cargo.toml`, `ferro-projection/Cargo.toml`, `ferro-projections/Cargo.toml`

**Files read:** 8 source files + 2 Cargo.toml manifests

**Key finding — `svc.fields.len()` in prop_map:** `ServiceDef.fields` is a public `Vec<FieldDef>` (service.rs line 69), so `svc.fields.len()` is accessible. However, since `.field()` is a consuming builder, the length must be captured before or via the iteration index — use `enumerate()` in the `prop_map` closure.

**Key finding — private SIGNAL_ constants:** The `SIGNAL_*` constants in `derive.rs` are private to that module. Integration tests in `catalog.rs` must use matching string literals. All constant values verified at derive.rs lines 9–47 and confirmed stable (they are the `matching_signals` values stored in `IntentScore`).

**Key finding — `has_one` navigation default:** `ServiceDef::has_one()` creates a `OneToOne` relationship with `NavigationHint::Inline` by default (confirmed: derive.rs line 1254–1270 shows `has_one("profile", "profile")` produces `inline_relationships` signal). The Focus fixture can use `.has_one(...)` directly.

**Pattern extraction date:** 2026-06-12
