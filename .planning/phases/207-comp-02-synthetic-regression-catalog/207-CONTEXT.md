# Phase 207: COMP-02 — Synthetic Regression Catalog - Context

**Gathered:** 2026-06-12 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a permanent, machine-checkable regression catalog in `ferro-projections/tests/catalog.rs`
that asserts `derive_intents()` produces the correct primary structural intent for each of the
seven canonical app classes (Browse / Focus / Collect / Process / Summarize / Analyze / Track),
plus structural-invariant assertions, proptest engine invariants, and adversarial competing-signal
fixtures. The catalog is the regression foundation for every future change to `derive.rs` /
`intent.rs` and the ground-truth source Phase 210's agent harness consumes.

**In scope:** test code only — canonical `ServiceDef` builders, derivation assertions, structural
invariants on `ServiceDef`/`IntentScore` output, `proptest` invariants, `insta` snapshots for named
canonical shapes, adversarial fixtures, CI integration, a discovered-weaknesses note.

**Out of scope (would be scope creep):** any change to `intent.rs` (the 7 intents) or `derive.rs`
scoring; new published crates; a visual/JSON-UI render path inside `ferro-projections`; the Phase 208
sketch renderers; gestiscilo migration (Phase 209); the agent harness itself (Phase 210).
</domain>

<decisions>
## Implementation Decisions

### Render-assertion strategy (SC#2 "structural property of rendered output")
- **D-01:** Assert the per-intent structural property **on the `ServiceDef` + `derive_intents()`
  output in-crate** — do NOT add a renderer dependency. `ferro-json-ui` depends on
  `ferro-projections` (optional, `Cargo.toml:24`); a reverse dev-dependency would create a cycle
  through a dev edge and violates the CLAUDE.md rule "do not add dependencies to ferro-projections"
  (`CLAUDE.md:9`). There is no concrete `Renderer` inside `ferro-projections` in Phase 207 (the
  `render/` module ships only the trait + `BaseContext`; sketch renderers arrive in Phase 208).
- **D-02:** Express SC#2's "Browse produces a table shape" as a **structural invariant on the
  fixture**: e.g. a Browse fixture exposes N column-bearing list/entity fields → assert the derived
  signals and the field/column count, not a rendered DOM. No test passes on an empty/minimal
  `ServiceDef` (SC#2 requirement) — every fixture is non-trivial.

### Snapshot tooling (insta)
- **D-03:** Add `insta` as a **dev-dependency** of `ferro-projections` (not present anywhere in the
  workspace yet). Use it ONLY for the seven named canonical shapes (SC#2: "insta snapshots only for
  named canonical shapes").
- **D-04:** Snapshot the **ranked `(intent, matching_signals)` list** per canonical fixture, with
  raw `confidence` floats redacted/rounded out of the snapshot to avoid fragility against future
  `derive.rs` tuning. Structural-invariant assertions MUST outnumber insta snapshot assertions
  (SC#2 — hard requirement).

### Proptest invariants
- **D-05:** `proptest` asserts **derivation-engine robustness invariants** over generated
  `ServiceDef`s, not specific intents: `derive_intents()` never panics, returns a non-empty ranked
  list, every `confidence ∈ [0.0, 1.0]`, the list is sorted descending by confidence, and contains
  no duplicate `Intent`. Build a bounded `Strategy<ServiceDef>` (random valid fields/meanings/
  relationships within sane size limits). `proptest` is already used in the workspace at version
  `"1"` (`ferro-reservation`, `ferro-projection`) — match that.

### Adversarial / competing-signal fixtures
- **D-06:** Provide a **competing-signal fixture per confusable intent pair**, each documented with
  a comment naming which signal should win and why: Browse↔Summarize, Process↔Track,
  Analyze↔Summarize, Collect↔Focus (≥4, scaling toward per-intent where a meaningful adversary
  exists). This satisfies the SC#3 floor ("at least one fixture is explicitly adversarial") and the
  deliverable's "adversarial fixture per intent" spirit, and serves the milestone honesty
  requirement (validation must be able to fail).

### Confidence-threshold assertions
- **D-07:** Each canonical test asserts (a) **hard primary-intent identity**
  (`derive_intents(&svc)[0].intent == Expected`), (b) a **margin** of the primary over the runner-up,
  and (c) a **conservative per-intent confidence floor calibrated below the first observed run** so
  genuine regression fails the gate without thrashing on benign `derive.rs` re-tuning. Calibrate the
  numeric floors/margins after a first real run of the fixtures — do not invent absolute numbers at
  plan time.

### Catalog organization & CI
- **D-08:** Single file `ferro-projections/tests/catalog.rs`: a fixtures module with 7 canonical
  `ServiceDef` builder functions, one `#[test]` per intent (identity + confidence + structural
  invariant), the adversarial tests, the snapshot tests, and the proptest. **No `#[ignore]`** — all
  tests run in the default `cargo test --all-features` CI gate (SC#4), so a future `derive.rs`
  regression produces a named, legible CI failure.

### Discovered-weaknesses note (deliverable gate)
- **D-09:** The phase verification MUST include a "discovered weaknesses" note naming ≥1 real
  limitation surfaced while writing the catalog (e.g. a canonical class with lower-than-expected
  derivation confidence, or a signal gap). An empty section fails phase close (SC#5).

### Claude's Discretion
- Exact builder-function names, fixture field sets, and the bounded proptest `Strategy` shape.
- Whether margin and floor are asserted as separate `assert!`s or a small helper.
- Snapshot file naming under `insta`'s convention.
</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & roadmap
- `.planning/REQUIREMENTS.md` — COMP-02 (line 14) defines the harness contract: structural
  invariants over byte snapshots, ≥1 competing-signal fixture. Non-goals table (lines 23–34)
  forbids touching `intent.rs`/`derive.rs` scoring and any rmcp upgrade.
- `.planning/ROADMAP.md` §"Phase 207: COMP-02 — Synthetic Regression Catalog" (line 2733) —
  Goal, Depends-on (nothing), and the five Success Criteria. Deliverable one-liner at line 2717.

### Intent / projection core (read, do NOT modify)
- `ferro-projections/src/intent.rs` — the 7-variant `Intent` enum, `IntentScore { intent,
  confidence, matching_signals }`, `IntentHint::{Primary,Exclude}`. Snake_case serde; `Custom`
  must stay last.
- `ferro-projections/src/derive.rs` — `derive_intents(service: &ServiceDef) -> Vec<IntentScore>`
  (line 75). Five analyzers, `BASELINE_BROWSE`/`BASELINE_FOCUS`, `normalize_scores` (ranked,
  descending), `apply_hints`, empty→`Focus@0.5` fallback. Grounds the confidence-floor calibration.
- `ferro-projections/src/service.rs` — `ServiceDef` fluent builder (`new`, `field`,
  `optional_field`, `list_field`, `read_only_field`, `write_only_field`, `action`, `guard`,
  `relationship`, `belongs_to`/`has_many`/`has_one`/`belongs_to_many`, `intent_hint`,
  `state_machine`, `validate`). The fixtures are built with this API.
- `ferro-projections/src/render/mod.rs` — `Renderer` trait + `BaseContext` only; no concrete
  renderer in-crate (grounds D-01/D-02).

### Test patterns & conventions
- `ferro-projections/tests/generate_schemas.rs` — existing integration-test layout and the exact
  public import set (`ActionDef, Cardinality, DataType, FieldDef, FieldMeaning, GuardDef,
  InputDef, Intent, IntentHint, IntentScore, NavigationHint, RelationshipDef, ServiceDef,
  StateDef, StateMachine, Transition, Warning`).
- `ferro-reservation/Cargo.toml:40` / `ferro-projection/Cargo.toml:31` — `proptest = "1"` precedent.
- `CLAUDE.md:9` — rendering-architecture rule: "do not add dependencies to ferro-projections"
  (the binding constraint behind D-01).
- `CLAUDE.md` Testing & Linting — CI runs `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets -- -D warnings`, `cargo test --all-features`.
</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ServiceDef` fluent builder (`service.rs:95`) — all seven canonical fixtures construct directly,
  no scaffolding needed.
- `derive_intents()` (`derive.rs:75`) — the single function under test; returns a ranked
  `Vec<IntentScore>` already sorted descending by `normalize_scores`.
- Public import surface proven in `generate_schemas.rs` — copy its `use ferro_projections::{...}`.
- `proptest = "1"` already vendored in the workspace lock via sibling crates.

### Established Patterns
- Integration tests live in `ferro-projections/tests/*.rs` (one file currently).
- `derive_intents` is total: even an empty service yields `Focus@0.5` — the proptest non-empty
  invariant (D-05) is guaranteed by design and worth pinning so it stays true.
- Browse and Focus carry derivation baselines — the most realistic false-positive risk, which is
  why Browse↔Summarize and Collect↔Focus are named adversarial pairs (D-06).

### Integration Points
- New crate-local dev-dependency: `insta` (D-03) — added to `ferro-projections/Cargo.toml`
  `[dev-dependencies]` (section does not exist yet; create it). `proptest = "1"` joins it.
- CI gate: tests must pass under `cargo test --all-features` with `-D warnings` clippy (no
  `#[ignore]`, no `#[allow]` shortcuts).
- Phase 210 (agent harness) consumes this catalog as ground truth — keep the canonical
  per-intent fixtures legible and addressable (stable builder names).
</code_context>

<specifics>
## Specific Ideas

- Snapshot content for D-04 is the ranked `(Intent, Vec<signal_name>)` projection, NOT raw floats —
  the named-canonical-shape artifact should survive benign confidence re-tuning.
- Adversarial fixtures (D-06) each carry an inline comment of the form
  `// competing: <signal A> vs <signal B>; <winner> must win because <reason>`.
- Confidence floors/margins (D-07) are filled in after a first `cargo test` run prints observed
  confidences — the plan should include that calibration step explicitly, not pre-bake numbers.
</specifics>

<deferred>
## Deferred Ideas

- Rendering a real JSON-UI spec from each canonical fixture and asserting on the rendered tree —
  belongs in `ferro-json-ui` (the crate that owns `JsonUiRenderer`), not in this catalog. Could be
  a future ferro-json-ui regression test that imports the catalog fixtures, but not Phase 207.
- Per-intent adversarial fixtures for all 7 intents (vs the confusable-pair set in D-06) — can be
  expanded later if the honesty review finds derivation weak spots the pair set misses.

None of the above are in Phase 207 scope.
</deferred>

---

*Phase: 207-comp-02-synthetic-regression-catalog*
*Context gathered: 2026-06-12*
