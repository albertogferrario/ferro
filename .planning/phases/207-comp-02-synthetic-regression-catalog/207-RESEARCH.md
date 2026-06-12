# Phase 207: COMP-02 — Synthetic Regression Catalog - Research

**Researched:** 2026-06-12
**Domain:** Rust integration testing — `ferro-projections` derivation engine (`derive_intents()`)
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Assert structural properties on `ServiceDef` + `derive_intents()` output in-crate. Do NOT add a renderer dependency. No reverse dev-dependency from `ferro-json-ui` (cycle risk + CLAUDE.md rule).
- **D-02:** SC#2 "Browse produces a table shape" expressed as a structural invariant on the fixture (field/column count assertion), not rendered DOM. Every fixture is non-trivial.
- **D-03:** Add `insta` as a dev-dependency of `ferro-projections` (not present anywhere in workspace yet). Used ONLY for the seven named canonical shapes.
- **D-04:** Snapshot the ranked `(intent, matching_signals)` list per canonical fixture, with raw `confidence` floats redacted/rounded out. Structural-invariant assertions MUST outnumber insta snapshot assertions.
- **D-05:** `proptest` asserts derivation-engine robustness invariants over generated `ServiceDef`s (never panics, non-empty, confidence ∈ [0,1], sorted descending, no duplicate Intent). Match workspace version `"1"`.
- **D-06:** Competing-signal fixture per confusable pair: Browse↔Summarize, Process↔Track, Analyze↔Summarize, Collect↔Focus (≥4). Each documented with `// competing: <A> vs <B>; <winner> must win because <reason>`.
- **D-07:** Each canonical test asserts (a) hard primary-intent identity, (b) margin of primary over runner-up, (c) conservative confidence floor calibrated AFTER a first real run. Do not pre-bake numbers.
- **D-08:** Single file `ferro-projections/tests/catalog.rs`. No `#[ignore]`. All tests run in `cargo test --all-features`.
- **D-09:** Verification MUST include a "discovered weaknesses" note naming ≥1 real limitation.

### Claude's Discretion

- Exact builder-function names, fixture field sets, and the bounded proptest `Strategy` shape.
- Whether margin and floor are asserted as separate `assert!`s or a small helper.
- Snapshot file naming under insta's convention.

### Deferred Ideas (OUT OF SCOPE)

- Rendering a real JSON-UI spec from each canonical fixture and asserting on the rendered tree — belongs in `ferro-json-ui`.
- Per-intent adversarial fixtures for all 7 intents beyond the confusable-pair set.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| COMP-02 | Synthetic catalog of canonical app classes covering the seven structural intents with regression harness on every `derive_intents()` change. Asserts structural invariants; includes at least one competing-signal fixture. | All research findings directly enable implementation: per-intent fixture recipes, proptest Strategy, insta snapshot idiom, adversarial pair rationale. |
</phase_requirements>

---

## Summary

Phase 207 creates `ferro-projections/tests/catalog.rs`: a single integration-test file that is the permanent regression foundation for the `derive_intents()` engine. The entire phase is test code — no production file may be modified. The system under test is well-understood: `derive_intents()` is a deterministic, synchronous, total function (always returns at least one score) that runs five analyzers and normalizes scores to `[0.0, 1.0]`.

The derivation logic has been read completely. Each analyzer's signal weights are known exactly (e.g., Money = 0.3 per field toward Summarize, EntityName = 0.2 toward Browse, `has_many` = 0.35 per relationship toward Browse, guarded transitions = 0.4 × guard_ratio toward Process). This lets us compute — by hand — the exact raw scores each fixture will produce, which informs the fixture design so the intended intent wins clearly and the adversarial pairs are genuinely competitive.

The two new dev-dependencies (`insta = "1"` and `proptest = "1"`) are both already in the workspace lock (`proptest 1.11.0`; `insta` not yet present but at `1.48.0` in the registry). The `[dev-dependencies]` section of `ferro-projections/Cargo.toml` does not yet exist and must be created.

**Primary recommendation:** Build each canonical fixture with a deliberate multi-signal set (3–5 domain fields, appropriate relationships/state machine/actions) so the primary intent wins by a margin that survives benign `derive.rs` re-tuning. Run a calibration pass (`cargo test -- --nocapture`) to observe raw confidence values before writing the numeric assertions.

---

## Architectural Responsibility Map

| Capability | Primary Tier | Secondary Tier | Rationale |
|------------|-------------|----------------|-----------|
| Intent derivation | `ferro-projections` (in-crate) | — | `derive_intents()` is pure Rust, synchronous, no I/O |
| Canonical `ServiceDef` fixtures | `ferro-projections/tests/` | — | Test code; no production surface |
| Structural invariant assertions | Test code in `catalog.rs` | — | Asserts on `Vec<IntentScore>` fields directly |
| `insta` snapshots | Test code in `catalog.rs` | `ferro-projections/tests/snapshots/` | insta writes `.snap` files next to tests by convention |
| `proptest` strategy | Test code in `catalog.rs` | — | Pure strategy over `ServiceDef` builder; no I/O |
| Adversarial fixtures | Test code in `catalog.rs` | — | Same module, documented with inline comment |
| CI gate | `cargo test --all-features` | `.github/workflows/publish.yml` | No extra config needed for integration tests |

---

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `proptest` | `"1"` (1.11.0 in lock) | Property-based testing of engine invariants | Already in workspace at this version (`ferro-reservation`, `ferro-projection`) [VERIFIED: Cargo.lock] |
| `insta` | `"1"` (1.48.0 in registry) | Snapshot testing for named canonical shapes | Rust snapshot testing de facto standard; version confirmed via `cargo search` [VERIFIED: cargo search] |

### Cargo.toml Addition Required

```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
proptest = "1"
```

`[dev-dependencies]` section does not exist yet in `ferro-projections/Cargo.toml`. [VERIFIED: file read]

The `yaml` feature is needed for `assert_yaml_snapshot!`. The `redactions` feature is optional but useful for redacting `confidence` floats inline (see D-04). If using a hand-rolled redacted struct instead, `yaml` alone suffices.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `insta` yaml snapshots | `assert_debug_snapshot!` | Debug format is less readable in diffs; yaml is cleaner for `Vec<(intent, signals)>` |
| `proptest` macro | Manual fuzzing loop | `proptest` shrinking is valuable for debugging failures; prefer it |

---

## Architecture Patterns

### System Architecture Diagram

```
catalog.rs (test file)
│
├── mod fixtures
│   ├── browse_catalog()    → ServiceDef  ─────────┐
│   ├── focus_detail()      → ServiceDef            │
│   ├── collect_form()      → ServiceDef            ▼
│   ├── process_workflow()  → ServiceDef    derive_intents(&svc)
│   ├── summarize_dash()    → ServiceDef            │
│   ├── analyze_timeseries()→ ServiceDef            ▼
│   └── track_timeline()    → ServiceDef    Vec<IntentScore>
│                                                   │
├── #[test] canonical_* (×7)    ◄──────────────────┤
│   assert [0].intent == X                          │ 
│   assert [0].confidence > floor                   │
│   assert margin([0] - [1])                        │
│   structural invariant (field count, etc.)        │
│   assert_yaml_snapshot!(redacted_signals)         │
│                                                   │
├── #[test] adversarial_* (×4)  ◄──────────────────┤
│   competing signal pairs:                         │
│   browse_vs_summarize, process_vs_track,          │
│   analyze_vs_summarize, collect_vs_focus          │
│                                                   │
└── proptest! engine_invariants  ◄─────────────────┘
    Strategy<ServiceDef> (random valid defs)
    assert: non-empty, confidence ∈ [0,1],
            sorted desc, no duplicate Intent
```

### Recommended Project Structure

```
ferro-projections/
├── Cargo.toml            — add [dev-dependencies]: insta, proptest
└── tests/
    ├── generate_schemas.rs   — existing (do not modify)
    ├── catalog.rs            — NEW (this phase)
    └── snapshots/            — auto-created by insta on first run
        ├── catalog__canonical_browse.snap
        ├── catalog__canonical_focus.snap
        └── ... (one per canonical test)
```

---

## Per-Intent Fixture Recipes

This is the core research artifact. Each recipe is derived from reading `derive.rs` analyzer weights directly.

### Critical Scoring Background

**Baselines:** Browse and Focus each receive +0.1 baseline before normalization. This means any intent other than Browse/Focus needs positive signals to overcome this baseline penalty. Normalization is `score / max_score`, so the baseline only matters relative to other intents' raw scores.

**System fields excluded:** `Identifier`, `CreatedAt`, `UpdatedAt`, `ForeignKey` are system fields (`is_system_field()` returns true for these). They do NOT contribute to field-meaning signals. All fixtures must include enough domain (non-system) fields to generate clear signals.

### Intent 1: Browse

**Target signals:** Entity navigation, collection structure (multiple EntityName/Category fields + has_many relationships + simple CRUD actions).

**Weight calculation:**
- 2× EntityName field: `2 × 0.2 = 0.4` Browse
- 2× Category field: `2 × 0.1 = 0.2` Browse
- 2× has_many relationship: `2 × 0.35 = 0.7` Browse
- Browse baseline: `+0.1` → raw Browse = `1.4`
- Simple CRUD action (no triggers/preconditions): `+0.05` Browse
- Focus baseline: `+0.1` → raw Focus ≈ `0.1` (only baseline)
- Normalize: Browse = `1.0`, Focus ≈ `0.07`

**Structural invariant:** Assert ≥2 domain fields with `FieldMeaning::EntityName` or `FieldMeaning::Category`; assert ≥1 `has_many` relationship.

**Fixture skeleton:**

```rust
// Source: derived from analyze_relationships() + analyze_field_meanings() in derive.rs
fn browse_catalog() -> ServiceDef {
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
```

### Intent 2: Focus

**Target signals:** Rich single-entity display (FreeText + ImageUrl + Url fields + inline/parent relationships).

**Weight calculation:**
- 2× FreeText: `2 × 0.25 = 0.5` Focus
- 1× ImageUrl: `1 × 0.25 = 0.25` Focus
- 1× Url: `1 × 0.25 = 0.25` Focus
- has_one (OneToOne, Inline): `1 × 0.15 = 0.15` Focus
- belongs_to (ManyToOne): `1 × 0.1 = 0.1` Focus
- Focus baseline: `+0.1` → raw Focus = `1.35`
- Browse baseline: `+0.1` → raw Browse ≈ `0.1`
- readable_count > writable_count → `+0.1` Focus (if mostly read-only)
- Normalize: Focus = `1.0`, Browse ≈ `0.07`

**Structural invariant:** Assert ≥2 fields with `FieldMeaning::FreeText`, `ImageUrl`, or `Url`.

**Fixture skeleton:**

```rust
fn focus_detail() -> ServiceDef {
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
```

### Intent 3: Collect

**Target signals:** High writable ratio (>50%) + write_only fields.

**Weight calculation:**
- 5× writable fields, 1× read-only (Identifier): ratio = 5/5 non-system = 100% writable → `+0.35` Collect
- 2× write_only field: `2 × 0.2 = 0.4` Collect → raw Collect = `0.75`
- Focus baseline: `0.1`, Browse baseline: `0.1`
- readable < writable → NO `more_readable` Focus signal
- Normalize: Collect = `1.0`, Browse/Focus ≈ `0.13`

**Structural invariant:** Assert non-system field count ≥ 4; assert majority of non-system fields are writable (`field.writable == true`).

**Fixture skeleton:**

```rust
fn collect_form() -> ServiceDef {
    ServiceDef::new("registration_form")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("name", DataType::String, FieldMeaning::EntityName)
        .field("email", DataType::String, FieldMeaning::Email)
        .field("phone", DataType::String, FieldMeaning::Phone)
        .write_only_field("password", DataType::String, FieldMeaning::Sensitive)
        .write_only_field("password_confirm", DataType::String, FieldMeaning::Sensitive)
        .field("terms_accepted", DataType::Boolean, FieldMeaning::Boolean)
}
```

### Intent 4: Process

**Target signals:** Guarded branching state machine + transition-trigger actions.

**Weight calculation (key signals):**
- 4 guarded transitions / 5 total → ratio 0.8 → `0.4 × 0.8 = 0.32` Process
- 2 branching states → `+0.15` Process
- 3 trigger actions / 3 total → `0.25 × 1.0 = 0.25` Process (state machine analyzer)
- 3 workflow actions → `3 × 0.15 = 0.45` Process (action analyzer)
- 3 non-final states > 2 → `+0.10` Process
- 3 guarded actions → `3 × 0.1 = 0.3` Process
- Browse baseline `0.1`, Focus baseline `0.1`
- Total raw Process >> other intents → Process = `1.0` after normalization
- Status field: `+0.25` Track — must be present but not dominate

**Structural invariant:** Assert state_machine is Some; assert ≥2 guarded transitions; assert ≥2 branching states (states with >1 outgoing transition).

**Fixture skeleton:** (use the order-management pattern already exemplified in `derive.rs` tests)

```rust
fn process_workflow() -> ServiceDef {
    // Uses the order lifecycle pattern already validated in derive.rs unit tests
    // Branching: draft → (pending | cancelled), pending → (approved | cancelled)
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
                .transition(Transition::new("draft", "submit", "submitted")
                    .guard("has_required_fields"))
                .transition(Transition::new("submitted", "approve", "approved")
                    .guard("is_approver"))
                .transition(Transition::new("submitted", "reject", "rejected")
                    .guard("is_approver"))
                .transition(Transition::new("draft", "cancel", "cancelled")
                    .guard("is_cancellable"))
                .transition(Transition::new("submitted", "cancel", "cancelled")
                    .guard("is_cancellable")),
        )
        .action(ActionDef::new("submit")
            .precondition("has_required_fields")
            .transition_trigger("submit"))
        .action(ActionDef::new("approve")
            .precondition("is_approver")
            .transition_trigger("approve"))
        .action(ActionDef::new("reject")
            .precondition("is_approver")
            .transition_trigger("reject"))
        .action(ActionDef::new("cancel")
            .precondition("is_cancellable")
            .transition_trigger("cancel"))
}
```

Note: `validate()` requires that guard names in `transition.guard` and `action.precondition` are declared via `.guard(GuardDef::new(...))`. The fixture above includes those declarations.

### Intent 5: Summarize

**Target signals:** Multiple read-only Money/Percentage/Quantity fields (dominant numeric aggregation). Must overcome Browse + Focus baselines.

**Weight calculation:**
- 3× Money (read-only): `3 × 0.3 = 0.9` Summarize (field meanings)
- 2× Percentage (read-only): `2 × 0.3 = 0.6` Summarize
- 1× Quantity (read-only): `1 × 0.3 = 0.3` Summarize
- Total from field meanings: `1.8` Summarize
- 5 non-system read-only / 5 non-system total = 83% > 70% → `+0.2` Summarize (writability)
- Browse baseline `0.1`, Focus baseline `0.1`
- `readable_count > writable_count` → `+0.1` Focus (but Browse raw = 0.1, Focus raw = 0.2, Summarize raw = 2.0)
- Normalize: Summarize = `1.0`, Focus ≈ `0.1`, Browse ≈ `0.05`

**Structural invariant:** Assert ≥3 non-system fields have `FieldMeaning::Money`, `Percentage`, or `Quantity`; assert all domain fields are non-writable.

**Fixture skeleton:**

```rust
fn summarize_dashboard() -> ServiceDef {
    ServiceDef::new("revenue_summary")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .read_only_field("total_revenue", DataType::Float, FieldMeaning::Money)
        .read_only_field("average_order", DataType::Float, FieldMeaning::Money)
        .read_only_field("gross_margin", DataType::Float, FieldMeaning::Percentage)
        .read_only_field("conversion_rate", DataType::Float, FieldMeaning::Percentage)
        .read_only_field("unit_count", DataType::Integer, FieldMeaning::Quantity)
        .read_only_field("return_rate", DataType::Float, FieldMeaning::Percentage)
}
```

### Intent 6: Analyze

**Target signals:** DateTime co-occurring with numeric measures. This is the hardest intent to isolate because the Analyze signal is a single joint condition (DateTime AND any Money/Percentage/Quantity co-occurrence → `+0.35`). Must beat Browse baseline.

**Weight calculation:**
- DateTime + Money co-occurrence → `+0.35` Analyze
- 3× Money (also contributes `3 × 0.3 = 0.9` Summarize — this is a strong competing signal)
- Browse baseline `0.1`, Focus baseline `0.1`
- `mostly_read_only` (>70% non-writable) → `+0.2` Summarize
- Raw: Summarize ≈ `1.1`, Analyze ≈ `0.35`, Browse ≈ `0.1`

**IMPORTANT FINDING:** With a purely "Money + DateTime" fixture, Summarize wins over Analyze (the datetime_numeric signal is only `0.35` raw vs Summarize's per-Money `0.3` × count + `0.2` mostly_read_only). To make Analyze win, the fixture must either:
1. Use many DateTime fields co-occurring with moderate numeric counts (so Analyze's `0.35` flat signal is proportionally large relative to Summarize), OR
2. Keep Money/Percentage/Quantity counts low (1 each) so Summarize raw ≈ 0.3 while Analyze raw = 0.35.

**Recommended approach:** 1× Money + 1× DateTime (no Percentage/Quantity) → Analyze raw = `0.35`, Summarize raw = `0.3 + 0.2(mostly_read_only if applicable)`. Without the mostly_read_only trigger, raw Analyze (0.35) > raw Summarize (0.30). Plus multiple DateTime fields to emphasize time-series character.

**Revised weight calculation for sparse fixture:**
- 1× Money: `0.3` Summarize
- 2× DateTime (has_datetime=true): Analyze co-occurrence fires once → `+0.35` Analyze
- Browse baseline `0.1`, Focus baseline `0.1`
- Non-writable ratio: if only 1 non-system writable field, non_writable may not cross 70% threshold
- Raw: Analyze = `0.35`, Summarize = `0.3`, Browse = `0.1` → Analyze wins (normalized 1.0)

**Structural invariant:** Assert ≥1 field with `FieldMeaning::DateTime`; assert Analyze signal `matching_signals` contains `"datetime_numeric_cooccurrence"`.

**Fixture skeleton:**

```rust
fn analyze_timeseries() -> ServiceDef {
    ServiceDef::new("sales_timeseries")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("recorded_at", DataType::DateTime, FieldMeaning::DateTime)
        .field("period_start", DataType::DateTime, FieldMeaning::DateTime)
        .field("period_end", DataType::DateTime, FieldMeaning::DateTime)
        .read_only_field("revenue", DataType::Float, FieldMeaning::Money)
        .read_only_field("created_at", DataType::DateTime, FieldMeaning::CreatedAt)
}
```

Note: CreatedAt is a system field (excluded from domain analysis). `recorded_at` and `period_start`/`period_end` with `FieldMeaning::DateTime` are domain fields that trigger `has_datetime = true`. Revenue triggers `has_numeric = true`. The `datetime_numeric_cooccurrence` signal fires once with weight `0.35`, beating Summarize's `0.30` from a single Money field.

### Intent 7: Track

**Target signals:** Linear state machine (no branching, no guards) + Status field.

**Weight calculation:**
- Status field: `0.25` Track
- 3 non-final states > 2: `+0.3` Track (linear_states signal) — requires branching_states == 0
- has_final_states: `+0.1` Track
- unguarded_progression: `+0.1` Track (guarded_count == 0)
- Browse baseline `0.1`, Focus baseline `0.1`
- Raw Track = `0.75`, Browse = `0.1`, Focus = `0.1`
- Normalize: Track = `1.0`, Browse ≈ `0.13`

**Structural invariant:** Assert state_machine is Some; assert no transition has a guard; assert state_machine has ≥3 non-final states; assert first score's `matching_signals` contains `"linear_states"`.

**Fixture skeleton:**

```rust
fn track_timeline() -> ServiceDef {
    ServiceDef::new("shipment_tracking")
        .field("id", DataType::Integer, FieldMeaning::Identifier)
        .field("tracking_number", DataType::String, FieldMeaning::EntityName)
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
                .transition(Transition::new("in_transit", "dispatch", "out_for_delivery"))
                .transition(Transition::new("out_for_delivery", "deliver", "delivered")),
        )
}
```

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Snapshot-based regression | Manual `assert_eq!(format!(...))` | `insta` `assert_yaml_snapshot!` | insta provides `cargo insta review` CLI, smart diffing, automatic snapshot creation on first run |
| Property-based testing | Manual random loop | `proptest!` macro | shrinking + replay corpus — a failing case is automatically minimized |
| Redacting floats from snapshots | Custom serialization | `insta` redaction `{".confidence" => "[float]"}` or hand-rolled struct | Simpler to maintain than custom Serialize impl |
| Computing engine invariants | Raw loop with custom error messages | `prop_assert!` / `prop_assert_eq!` | On failure, proptest prints the minimal counterexample |

**Key insight:** The snapshot tool must survive `confidence` value drift from `derive.rs` re-tuning. Redacting or omitting `confidence` from the snapshot payload (D-04) is the correct isolation strategy — use a dedicated struct that only captures `(Intent, Vec<String>)`.

---

## Confidence Calibration Protocol (D-07)

**CRITICAL: Do not invent numeric thresholds.** The plan must include an explicit calibration task:

1. Implement all 7 canonical fixtures with `assert_eq!([0].intent == Expected)` only (no numeric assertions yet).
2. Run `cargo test -p ferro-projections -- --nocapture 2>&1 | grep confidence` OR add a temporary `eprintln!` inside each test printing `derive_intents(&svc)[0].confidence`.
3. Record the observed primary confidence per intent and the runner-up confidence.
4. Set `confidence_floor` = `observed_primary - 0.15` (conservative: 15 percentage-point cushion).
5. Set `margin_floor` = `observed_primary - observed_runner_up - 0.10` (10pp cushion below observed margin).
6. Remove the temporary debug prints; add the numeric assertions.

This protocol ensures the gate is calibrated to reality, not to a guess. The plan must list this calibration as an explicit plan step, not as part of the same task that writes fixtures.

---

## Adversarial Pair Analysis (D-06)

### Pair 1: Browse ↔ Summarize

**Competing signals:** A "rich product catalog" has EntityName/Category fields (Browse) AND Money fields (Summarize). The fixture must make Summarize win despite Browse's baseline.

**Setup:**
- 1× EntityName: `0.2` Browse
- 2× Category: `0.2` Browse
- 3× Money: `0.9` Summarize
- Browse baseline: `+0.1` → raw Browse = `0.5`, raw Summarize = `0.9`
- **Summarize wins** because per-field Money signal (`0.3/field`) dominates EntityName+Category vs baseline.

**Comment:** `// competing: entity_name+category (Browse baseline 0.5) vs money_fields (Summarize 0.9); Summarize must win because per-field money signal outweighs entity_name accumulation`

### Pair 2: Process ↔ Track

**Competing signals:** A state machine with mixed guarded + unguarded transitions. The guard density determines the winner.

**Setup for Process win:** ≥3 guarded / 4 total transitions → `0.4 × 0.75 = 0.30` Process + branching → `+0.15` Process. Track gets `0.10` has_final_states but NOT linear_states (branching exists) and NOT unguarded_progression (guards exist). Process raw >> Track raw.

**Setup for Track win (alternative adversarial):** 0 guards, 4 transitions, 3 non-final states → Track = `0.25(status) + 0.3(linear) + 0.1(final) + 0.1(unguarded) = 0.75`. Process gets no signals. This is the standard Track fixture — the adversarial is the guard-mixed case above.

**Comment:** `// competing: guarded_transitions (Process) vs linear_states+unguarded (Track); Process must win because branching factor + guard density dominates Track's linear signal`

### Pair 3: Analyze ↔ Summarize

**Competing signals:** Time-series data with multiple Money fields. As analyzed above, Summarize wins with many Money fields. The adversarial fixture uses the reverse: many DateTime fields with sparse Money to let Analyze win.

**Setup (Analyze wins):** 3× DateTime domain fields + 1× Money → Analyze raw = `0.35` (co-occurrence fires once), Summarize raw = `0.30` (one Money field). Analyze wins narrowly.

**Comment:** `// competing: datetime_numeric_cooccurrence (Analyze 0.35) vs money_fields (Summarize 0.30); Analyze must win because temporal density outweighs single monetary measure in time-series context`

### Pair 4: Collect ↔ Focus

**Competing signals:** A detailed user profile form has FreeText/ImageUrl fields (Focus) AND many writable fields (Collect). The fixture must make Collect win.

**Setup:**
- 1× FreeText: `0.25` Focus
- 1× ImageUrl: `0.25` Focus → raw Focus signal = `0.50` from fields
- 5× writable non-system fields (>50% writable): `+0.35` Collect
- 2× write_only: `2 × 0.2 = 0.40` Collect → raw Collect = `0.75`
- Focus baseline `0.1`, write-only fields also mean readable < writable → no `more_readable` Focus boost
- Focus raw ≈ `0.60` (0.50 + 0.10 baseline), Collect raw = `0.75`
- **Collect wins** because write_only fields' Collect signal exceeds FreeText/ImageUrl Focus signal.

**Comment:** `// competing: free_text+image_url (Focus ~0.60) vs high_writable_ratio+write_only (Collect 0.75); Collect must win because write-only credential fields accumulate 0.4 on top of writable ratio signal`

---

## proptest Strategy Design (D-05)

The proptest `Strategy<ServiceDef>` must be bounded and valid (no state-machine validation failures that would require `validate()` to succeed — the proptest only tests `derive_intents()` which is infallible).

**Key design:** Since `derive_intents()` is total and doesn't call `validate()`, the strategy can generate structurally arbitrary (not necessarily valid) ServiceDefs. This simplifies the strategy significantly.

```rust
// Source: derive from proptest::prelude::* idiom in ferro-projection/tests/proptest_properties.rs
use proptest::prelude::*;

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
    // 0–8 non-system fields; no state machine (simplest valid form)
    proptest::collection::vec(
        (arb_data_type(), arb_field_meaning()),
        0..8usize,
    )
    .prop_map(|fields| {
        let mut svc = ServiceDef::new("proptest_subject");
        for (dt, meaning) in fields {
            svc = svc.field(format!("f_{}", svc.fields.len()), dt, meaning);
        }
        svc
    })
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
        // Invariant 2: all confidence in [0.0, 1.0]
        for s in &scores {
            prop_assert!(s.confidence >= 0.0, "confidence below 0: {}", s.confidence);
            prop_assert!(s.confidence <= 1.0, "confidence above 1: {}", s.confidence);
        }
        // Invariant 3: sorted descending
        for i in 1..scores.len() {
            prop_assert!(
                scores[i-1].confidence >= scores[i].confidence,
                "not sorted at [{i}]: {} < {}",
                scores[i-1].confidence, scores[i].confidence
            );
        }
        // Invariant 4: no duplicate Intent (use debug string for comparison since Intent: !Hash is not needed)
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
```

**Note on Intent and Hash:** `Intent` derives `Hash` (verified in `intent.rs` line 13: `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]`). The no-duplicate invariant can use a `HashSet<String>` or directly a `HashSet` after collecting intents. [VERIFIED: intent.rs]

---

## Snapshot Idiom for D-04

The snapshot must capture `(intent_name, matching_signals)` without `confidence` floats. Build a redacted view struct:

```rust
// Source: insta docs + Context7 /mitsuhiko/insta
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

#[test]
fn snapshot_canonical_browse() {
    let svc = fixtures::browse_catalog();
    let scores = derive_intents(&svc);
    insta::assert_yaml_snapshot!("canonical_browse", redacted_signals(&scores));
}
```

Snapshot file name is `catalog__canonical_browse.snap` by insta's convention (module path + test name). The first run with `INSTA_UPDATE=new` or `cargo insta review` creates the snapshot file. [VERIFIED: Context7 /mitsuhiko/insta]

**insta Cargo.toml entry:**

```toml
[dev-dependencies]
insta = { version = "1", features = ["yaml"] }
proptest = "1"
```

The `redactions` feature is optional — the hand-rolled `IntentSignals` struct is simpler and avoids the feature dependency.

---

## Common Pitfalls

### Pitfall 1: Validate() Failure Breaks Fixture Tests

**What goes wrong:** `ServiceDef::validate()` returns `Err` for undefined guard references. If a fixture uses `.guard()` name in a `.transition(...).guard("name")` or `.precondition("name")` without a matching `.guard(GuardDef::new("name"))`, the fixture is malformed. The `derive_intents()` function itself does NOT call `validate()` — but the plan should include a validation call per fixture to ensure no structural bugs in the canonical fixtures.

**How to avoid:** Every Process/Track fixture that uses state machines with guards MUST declare all guards via `.guard(GuardDef::new(...))`. The test should assert `svc.validate().is_ok()` before asserting derivation results.

**Warning signs:** `validate()` returning `Err` with "references undefined guard" message.

### Pitfall 2: Browse Baseline Traps Summarize/Analyze Fixtures

**What goes wrong:** Because Browse and Focus each get a `+0.1` baseline, a weak Summarize or Analyze fixture (few fields) may not beat Browse. The basket of signals must be large enough that the primary intent's raw score comfortably exceeds `0.1`.

**How to avoid:** Use ≥3 Money/Percentage/Quantity fields for Summarize; keep Money count low (1–2 fields) for Analyze fixtures where DateTime co-occurrence should win.

### Pitfall 3: Analyze Always Loses to Summarize

**What goes wrong:** Adding many Money fields to an Analyze fixture to "make it look financial" causes Summarize raw = `0.3 × n_money` which quickly exceeds Analyze's flat `0.35` signal. With 2 Money fields, Summarize raw = `0.6 + 0.2(mostly_read_only)` = `0.8` >> Analyze `0.35`.

**How to avoid:** Keep the Analyze fixture lean on Money/Percentage/Quantity (exactly 1 numeric field). Add multiple DateTime-meaning domain fields. Do NOT use `mostly_read_only` conditions in the Analyze fixture.

### Pitfall 4: insta Snapshot Drift in CI

**What goes wrong:** If `confidence` floats are included in the snapshot, any `derive.rs` re-tuning (even benign) breaks CI with a snapshot mismatch that requires `cargo insta review` — a human step that can't run in CI without `INSTA_UPDATE`.

**How to avoid:** Never include raw `confidence` floats in the snapshot payload. Use the `IntentSignals` redacted struct (signals only, no confidence). [D-04 is the binding rule]

### Pitfall 5: No Signals From System-Only Fixture

**What goes wrong:** A ServiceDef with only `Identifier`/`CreatedAt`/`UpdatedAt` fields produces only Browse + Focus at equal confidence (both baselines, tie-broken by priority). This satisfies SC#1's primary identity assertion trivially (Browse wins by tie-break, not by signals), which is weaker than the SC#2 requirement that "no test is satisfied by an empty or minimal ServiceDef."

**How to avoid:** Every fixture must have ≥3 non-system domain fields producing positive signals for the intended intent.

### Pitfall 6: State Machine Without Guard Declarations Causes CI Failure

**What goes wrong:** Calling `.transition(...).guard("name")` but not declaring `.guard(GuardDef::new("name"))` causes `validate()` to fail. While `derive_intents()` does not call validate, the catalog plan should validate fixtures as part of structural invariants.

**How to avoid:** The Process and Track fixtures must be validated clean (see Pitfall 1). Alternatively, omit guard declarations from state machine transitions for the Track fixture (Track specifically requires *unguarded* transitions, so guards are not needed there).

---

## Code Examples

### Pattern: Structural Invariant Helper

```rust
// Source: pattern derived from derive.rs unit test helpers (has_signal, find_intent)
fn primary_intent(svc: &ServiceDef) -> &IntentScore {
    let scores = ferro_projections::derive_intents(svc);
    assert!(!scores.is_empty(), "derive_intents must not return empty");
    // SAFETY: scores is non-empty; return by value is fine in tests
    scores.into_iter().next().unwrap()
}

// Canonical test pattern
#[test]
fn canonical_browse() {
    let svc = fixtures::browse_catalog();
    let scores = ferro_projections::derive_intents(&svc);

    // (a) Hard primary identity
    assert_eq!(scores[0].intent, Intent::Browse, "Browse must be primary");

    // (b) Confidence floor (filled after calibration run)
    assert!(scores[0].confidence >= BROWSE_FLOOR,
        "Browse confidence {} below floor {}", scores[0].confidence, BROWSE_FLOOR);

    // (c) Margin over runner-up
    if scores.len() > 1 {
        assert!(scores[0].confidence - scores[1].confidence >= BROWSE_MARGIN,
            "Browse margin too narrow: {} vs {}", scores[0].confidence, scores[1].confidence);
    }

    // Structural invariant: ≥2 EntityName/Category domain fields
    let entity_fields = svc.fields.iter()
        .filter(|f| matches!(f.meaning, FieldMeaning::EntityName | FieldMeaning::Category))
        .count();
    assert!(entity_fields >= 2, "Browse fixture needs ≥2 entity/category fields");

    // Structural invariant: ≥1 has_many relationship
    use ferro_projections::Cardinality;
    let collection_rels = svc.relationships.iter()
        .filter(|r| matches!(r.cardinality, Cardinality::OneToMany | Cardinality::ManyToMany))
        .count();
    assert!(collection_rels >= 1, "Browse fixture needs ≥1 collection relationship");
}
```

### Pattern: Import Surface (copy from generate_schemas.rs)

```rust
use ferro_projections::{
    ActionDef, Cardinality, DataType, FieldDef, FieldMeaning, GuardDef, InputDef,
    Intent, IntentHint, IntentScore, NavigationHint, RelationshipDef, ServiceDef,
    StateDef, StateMachine, Transition, Warning,
};
use ferro_projections::derive_intents;
```

[VERIFIED: ferro-projections/tests/generate_schemas.rs line 4-8]

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| No regression catalog | `catalog.rs` with 7 canonical fixtures | Phase 207 | Future `derive.rs` changes must maintain per-intent correctness |
| Confidence values unchecked | Floor + margin assertions post-calibration | Phase 207 | Benign re-tuning won't break CI; regressions will |
| No property tests on engine | proptest over `arb_service_def()` | Phase 207 | Engine panics or invariant violations surface automatically |

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | The Analyze fixture (1 Money + 2 DateTime domain fields) produces Analyze > Summarize (raw 0.35 > 0.30) | Per-Intent Fixture Recipes — Analyze | Analyze fixture produces Summarize as primary; calibration run will reveal this and the fixture must add more DateTime fields or fewer Money fields |
| A2 | `insta 1.48.0` is the current stable version | Standard Stack | Minor version mismatch; `"1"` spec will resolve to actual latest, which is fine |
| A3 | The Collect↔Focus adversarial pair (1 FreeText + 1 ImageUrl + 2 write_only fields) produces Collect > Focus (0.75 > 0.60) | Adversarial Pair Analysis | If Focus Focus baseline + FreeText/ImageUrl sum exceeds Collect, swap fixture ratios; calibration will detect |

**If this table is non-empty:** The three assumptions above are weight-arithmetic estimates based on reading `derive.rs`. All three will be confirmed or falsified by the calibration run (Plan step: run tests with `--nocapture`, observe actual confidences, adjust fixture if needed before writing numeric assertions).

---

## Open Questions

1. **Analyze fixture confidence margin over Summarize**
   - What we know: Analyze raw = 0.35 flat, Summarize raw = 0.3 × 1 Money = 0.30 (without mostly_read_only)
   - What's unclear: Whether the calibration run confirms a clear enough margin (≥0.10 after normalization) for the margin assertion to be stable
   - Recommendation: If margin is thin (< 0.15 normalized), reduce Money fields to 0 and rely on a non-Money numeric (Quantity × 1) to preserve the numeric co-occurrence trigger while minimizing Summarize weight

2. **proptest cases count**
   - What we know: `ferro-projection` uses 32 cases; `derive_intents` is synchronous and fast
   - What's unclear: 256 cases may add noticeable time to `cargo test` on cold runs
   - Recommendation: Start at 256; if CI takes >5 seconds for the proptest alone, reduce to 128

---

## Environment Availability

Step 2.6: SKIPPED (no external dependencies; this phase is test code only within the existing Rust workspace; `cargo test --all-features` is the sole execution environment and is already confirmed operational per STATE.md).

---

## Validation Architecture

> `workflow.nyquist_validation` key absent in `.planning/config.json` → treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test + `proptest 1.11.0` + `insta 1.48.0` |
| Config file | None — standard `cargo test` discovery; `[dev-dependencies]` added to `ferro-projections/Cargo.toml` |
| Quick run command | `cargo test -p ferro-projections --test catalog` |
| Full suite command | `cargo test --all-features` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| COMP-02 SC#1 | `derive_intents(&svc)[0].intent == Expected` for all 7 intents | unit-style integration | `cargo test -p ferro-projections --test catalog -- canonical_` | ❌ Wave 0 |
| COMP-02 SC#2 | Structural invariant assertions outnumber snapshots | integration | same as above | ❌ Wave 0 |
| COMP-02 SC#2 | insta snapshots for 7 named canonical shapes | snapshot | `cargo test -p ferro-projections --test catalog -- snapshot_` | ❌ Wave 0 |
| COMP-02 SC#3 | Adversarial fixtures resolve competing signals correctly | integration | `cargo test -p ferro-projections --test catalog -- adversarial_` | ❌ Wave 0 |
| COMP-02 SC#4 | All tests pass under `cargo test --all-features`, no `#[ignore]` | CI gate | `cargo test --all-features` | ❌ Wave 0 |
| COMP-02 SC#5 | Discovered weaknesses note (verified in phase verification doc) | manual | `gsd-verify-work 207` | ❌ Wave 0 |
| proptest | Engine invariants (non-empty, sorted, confidence ∈ [0,1], no duplicates) | property-based | `cargo test -p ferro-projections --test catalog -- engine_` | ❌ Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p ferro-projections --test catalog`
- **Per wave merge:** `cargo fmt --all -- --check && cargo clippy --all --all-targets -- -D warnings && cargo test -p ferro-projections`
- **Phase gate:** `cargo test --all-features` full green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `ferro-projections/tests/catalog.rs` — the entire deliverable; covers all COMP-02 SCs
- [ ] `ferro-projections/Cargo.toml` `[dev-dependencies]` section — `insta = { version = "1", features = ["yaml"] }` + `proptest = "1"`
- [ ] `ferro-projections/tests/snapshots/` directory (auto-created by insta on first run with `INSTA_UPDATE=new`)

---

## Security Domain

> Skipped. This phase adds only test code to `ferro-projections`. No authentication, session management, input validation of untrusted data, cryptography, or external service calls. ASVS categories V2–V6 are not applicable.

---

## Sources

### Primary (HIGH confidence)

- `ferro-projections/src/derive.rs` — complete read; all analyzer signal weights, baseline constants, normalization logic verified
- `ferro-projections/src/intent.rs` — complete read; `Intent`, `IntentScore`, `IntentHint` types, derive macros, Hash impl
- `ferro-projections/src/service.rs` — complete read; all builder methods, field access flags, validate() rules
- `ferro-projections/Cargo.toml` — verified: no `[dev-dependencies]` section exists
- `ferro-projections/tests/generate_schemas.rs` — verified: exact public import surface
- `/mitsuhiko/insta` (Context7) — `assert_yaml_snapshot!`, `assert_debug_snapshot!`, Cargo.toml entry, redaction idiom
- `ferro-reservation/Cargo.toml` + `ferro-projection/Cargo.toml` — verified: `proptest = "1"` precedent
- `Cargo.lock` — verified: `proptest 1.11.0` in workspace lock; `insta` absent

### Secondary (MEDIUM confidence)

- `cargo search insta` output — current insta version `1.48.0` [VERIFIED: registry]
- `ferro-projection/tests/proptest_properties.rs` + `ferro-reservation/tests/property_invariants.rs` — proptest workspace idiom: `use proptest::prelude::*`, `proptest! { #![proptest_config(...)] #[test] fn name(...) { ... } }`

### Tertiary (LOW confidence)

- Analyze fixture confidence margin over Summarize (0.35 vs 0.30) — estimated from weight arithmetic; must be confirmed by calibration run [ASSUMED: A1]

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — proptest version verified in lockfile; insta version verified via cargo search
- Architecture: HIGH — all five analyzers read completely; per-intent weight calculations are deterministic
- Pitfalls: HIGH — derive.rs unit tests and integration tests already document the exact edge cases (system fields excluded, baseline behavior, empty service)

**Research date:** 2026-06-12
**Valid until:** 2026-12-12 (stable domain — `derive.rs` is the system under test and is not modified in this phase)
