# Phase 173 — Verification Record

**Phase:** 173-make-json-view-v2-projection-roundtrip-test
**Plans:** 173-01 (make:json-view ServiceDef pipeline), 173-02 (projection-roundtrip test)
**Status:** Automated gates passed; D-07 manual gate open (see below)

---

## SC1 — component_schema Role (Vacuous, Satisfied by Design)

**Criterion:** `catalog.component_schema()` is used for per-component structured output.

**Resolution:** SC1 is satisfied by design. The deterministic builder
(`Spec::from_service_def` in `ferro-json-ui/src/projection/builder.rs`) selects
components via `FieldMeaning` → component dispatch (`lookup_meaning` in
`component_map.rs`) and validates the complete produced spec against
`catalog.json_schema()` (SC2). `catalog.component_schema()` is the per-component
schema for LLM-constrained generation — it is used when an LLM is being prompted
to produce props for a named component. Since the deterministic builder performs
no per-component LLM call, no per-component schema prompt is needed. No LLM pass
was added to satisfy SC1.

The write-gate catalog validation (`catalog.json_schema()`) exercised by both
`make_json_view.rs` (`Spec::from_json` re-parse) and the roundtrip test
(`global_catalog().validate`) covers the full spec shape, making the
per-component `component_schema()` path redundant in the deterministic pipeline.

**Decision reference:** D-05 (CONTEXT.md) — "If the deterministic path needs no
per-component LLM call, SC1's component_schema() clause is satisfied vacuously
and the planner documents that in VERIFICATION.md rather than inventing an LLM
pass to use it."

---

## SC2 — Catalog Write-Gate

**Criterion:** Generated spec validates against `catalog.json_schema()` before write.

**Status:** Satisfied by two independent mechanisms:

1. `ferro-cli/src/commands/make_json_view.rs` — the `render_service_def` helper
   serializes the spec to JSON then calls `Spec::from_json(&json_str)` (re-parse
   write-gate, D-02); a parse failure falls back to the static template with a
   warning. Additionally, `Spec::from_service_def` itself calls
   `global_catalog().validate` before returning (D-06 contract in builder.rs).

2. `ferro-ai/tests/projection_roundtrip.rs` — explicitly calls
   `global_catalog().validate(&spec)` after `Spec::from_service_def` and asserts
   `is_ok()`.

---

## SC3 — Component Selection via FieldMeaning/Intent (Deterministic, Not LLM)

**Criterion:** Component selection is driven by `FieldMeaning`/`Intent` via
`Spec::from_service_def`, not by LLM re-prompting.

**Status:** Satisfied. The command `make:json-view` routes all spec generation
through `derive_intents(&service)` → `Spec::from_service_def(&service, &intents, &ctx)`.
No LLM call is made for component selection. The NL→ServiceDef stage (via
`scaffold_core`) determines *what* the service is; the deterministic renderer
determines *how it renders*.

**Automated gate:** `cargo test -p ferro-ai --test projection_roundtrip`

---

## SC4 — No v1 JsonUiView Types in the Pipeline

**Criterion:** The `make:json-view` pipeline contains no v1 `JsonUiView` types.

**Status:** Satisfied. Grep audit:

```
grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs
```

Expected result: 0. v1 types were deleted in Phase 160. The command uses `Spec`
(v2), `Spec::from_json` (v2 parse), and `global_catalog().validate` (v2 catalog)
throughout.

---

## SC5 — Roundtrip Passes via ServiceDef-Aware Path (Path-Proof)

**Criterion:** The roundtrip test cannot pass via a generic schema-normalization
fallback; it must go through the ServiceDef-aware `FieldMeaning` dispatch.

**Status:** Satisfied. The test at `ferro-ai/tests/projection_roundtrip.rs`
seeds a `ServiceDef` with a `FieldMeaning::Money` field (`total`, `DataType::Float`).
`Spec::from_service_def` deterministically maps this to a DataTable column with
`ColumnFormat::Currency` via `build_column_for_field` in `component_map.rs` (line 277):

```rust
FieldMeaning::Money => Some(ColumnFormat::Currency),
```

The test asserts:
```rust
let has_currency = cols.iter()
    .any(|c| c.get("format").and_then(|f| f.as_str()) == Some("currency"));
assert!(has_currency, "Money field must produce a currency-formatted column ...");
```

This assertion cannot pass via any path that does not execute the
`FieldMeaning::Money → ColumnFormat::Currency` dispatch — a generic NL→spec LLM
call would not reliably produce the `"currency"` format string.

**Automated gate:** `cargo test -p ferro-ai --test projection_roundtrip`

---

## D-07 Manual Gate — Live NL→ServiceDef Quality

**Disposition:** Open (not blocking the automated phase gate).

**Requirement:** Verify that `ferro make:json-view <name> --description "<NL>"` against
a configured `FERRO_AI_*` provider produces a view that reflects the description
and selects sensible components.

**Instructions:**

1. Configure an AI provider (e.g., `FERRO_AI_API_KEY=<key> FERRO_AI_MODEL=claude-3-5-haiku-20241022`).
2. Run: `ferro make:json-view orders --description "A list of customer orders showing order number, total amount, and delivery status"`
3. Inspect the generated `src/views/orders.json`:
   - The root element should be a `DataTable`.
   - Column names should reflect the description's fields (order number, total, status).
   - The `total` column, if typed as Money by `scaffold_core`, should carry `"format": "currency"`.
4. Sign off: `[ ] Confirmed — <date> <initials>`

**Precedent:** Mirrors Phase 171 SC4/SC6 live quality manual gate pattern.
The automated roundtrip test exercises the deterministic ServiceDef→Spec path.
The live quality gate confirms that the NL→ServiceDef (`scaffold_core`) stage
produces semantically sensible `ServiceDef` values for real descriptions.

---

## Summary

| SC | Status | Gate |
|----|--------|------|
| SC1 (component_schema) | Satisfied by design — vacuous, no LLM pass | Documented above |
| SC2 (catalog write-gate) | Satisfied | Automated (make_json_view.rs + roundtrip test) |
| SC3 (deterministic component selection) | Satisfied | `cargo test -p ferro-ai --test projection_roundtrip` |
| SC4 (no JsonUiView) | Satisfied | `grep -c "JsonUiView" ferro-cli/src/commands/make_json_view.rs` == 0 |
| SC5 (path-proof: currency assertion) | Satisfied | `cargo test -p ferro-ai --test projection_roundtrip` |
| D-07 (live NL quality) | Open — manual gate | See instructions above |
