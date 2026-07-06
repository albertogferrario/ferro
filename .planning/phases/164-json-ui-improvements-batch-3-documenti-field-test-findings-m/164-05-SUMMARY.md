---
phase: 164
plan: "05"
subsystem: ferro-json-ui
tags: [card-variant, json-ui, render, component, d-18, v7-runtime-friction]
dependency_graph:
  requires: [164-03]
  provides: [CardVariant, CardProps.variant, render_card-variant-branch]
  affects: [ferro-json-ui/src/component.rs, ferro-json-ui/src/render/containers.rs, ferro-json-ui/src/lib.rs, ferro-json-ui/src/projection/builder.rs]
tech_stack:
  added: [CardVariant enum (Bordered/Elevated), serde snake_case on new enum]
  patterns: [match-on-variant for CSS class selection, #[serde(default)] for backward compat]
key_files:
  created: []
  modified:
    - ferro-json-ui/src/component.rs
    - ferro-json-ui/src/render/containers.rs
    - ferro-json-ui/src/lib.rs
    - ferro-json-ui/src/projection/builder.rs
decisions:
  - "Used snake_case NOT lowercase (Pitfall 2 from RESEARCH) — single-word variants serialize identically today but workspace convention is snake_case for forward compat"
  - "variant field uses #[serde(default)] so all existing Card specs (no variant key) continue to render as Bordered — backward compat by construction"
  - "Elevated has no border (no border-border class) and uses p-8 padding — matches auth/error page aesthetic from V7-RUNTIME-FRICTION F10 spec"
  - "CardVariant::Bordered is the #[default] variant — dashboard density maintained"
metrics:
  duration_minutes: 25
  completed: "2026-05-17T02:20:00Z"
  tasks_completed: 3
  files_modified: 4
---

# Phase 164 Plan 05: CardVariant — Card chrome variant for auth/error/marketing pages

## One-liner

`CardVariant` enum (Bordered default, Elevated) on `CardProps` lets auth/error pages declare `"variant": "elevated"` to get `shadow-md + p-8` without border, while all existing dashboard Card specs keep their current `border + shadow-sm + p-4` chrome unchanged.

## What was built

Closes V7-RUNTIME friction **F10 / D-18**: Phase 162 D-05 removed the `AuthLayout`'s implicit `<div class="bg-card shadow-md p-8">` wrapper, and gestiscilo's `auth/login.json` + `errors/error.json` now declare a `Card` root that renders with dashboard chrome (border + shadow-sm + p-4). Auth/error/marketing pages need an elevated variant.

### New type in `ferro-json-ui/src/component.rs`

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CardVariant {
    #[default]
    Bordered,
    Elevated,
}
```

- `Bordered` (default): `rounded-lg border border-border bg-card shadow-sm overflow-visible` + `p-4`
- `Elevated`: `rounded-lg bg-card shadow-md overflow-visible` + `p-8` (no border)

### Updated `CardProps` in `ferro-json-ui/src/component.rs`

Added `#[serde(default)] pub variant: CardVariant` field. All existing CardProps struct literals in tests updated to include `variant: CardVariant::Bordered`.

### Updated renderer in `ferro-json-ui/src/render/containers.rs`

Replaced the hard-coded outer class string with a variant match:

```rust
let (outer_class, inner_pad) = match props.variant {
    CardVariant::Bordered => (
        "rounded-lg border border-border bg-card shadow-sm overflow-visible",
        "p-4",
    ),
    CardVariant::Elevated => ("rounded-lg bg-card shadow-md overflow-visible", "p-8"),
};
let mut html = format!("<div class=\"{outer_class}\"><div class=\"{inner_pad}\">");
```

### Re-export in `ferro-json-ui/src/lib.rs`

`CardVariant` added to the `pub use component::{...}` block.

## Tests added

### ferro-json-ui/src/component.rs — 6 serde/default tests (module `card_variant_tests`)

| Test | Purpose |
|------|---------|
| `card_variant_default_is_bordered` | `CardVariant::default() == Bordered` |
| `card_variant_serializes_snake_case` | Bordered → `"bordered"`, Elevated → `"elevated"` |
| `card_variant_deserializes_snake_case` | `"bordered"` → Bordered, `"elevated"` → Elevated |
| `card_props_without_variant_defaults_to_bordered` | JSON with no `variant` key → Bordered |
| `card_props_with_elevated_variant` | JSON `"variant": "elevated"` → Elevated |
| `card_props_roundtrip_preserves_variant` | Elevated round-trips through serde |

### ferro-json-ui/src/render/containers.rs — 3 render tests

| Test | Purpose |
|------|---------|
| `render_card_bordered_default` | Default Card emits `border border-border`, `shadow-sm`, `p-4` |
| `render_card_elevated_no_border` | Elevated Card emits `shadow-md`, `p-8`, NO `border-border` |
| `render_card_omitted_variant_is_bordered` | No variant field → Bordered chrome (backward compat) |

## Call sites updated (deviation — Rule 1/2 auto-fix)

| File | Line | Change |
|------|------|--------|
| `ferro-json-ui/src/component.rs` | 1241, 1266 | Two `CardProps` struct literals in `schema_smoke_tests` — added `variant: CardVariant::Bordered` |
| `ferro-json-ui/src/projection/builder.rs` | 342 | `CardProps` struct literal in projection builder — added `variant: CardVariant::Bordered`, imported `CardVariant` |

These call sites did not use `..Default::default()`, so the new non-optional field required explicit additions. All additions use `CardVariant::Bordered` to preserve existing rendering behavior (dashboard projection cards stay Bordered).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] projection/builder.rs missing variant field**
- **Found during:** Task 3 (clippy gate)
- **Issue:** `ferro-json-ui/src/projection/builder.rs:342` constructs `CardProps { ... }` without the new `variant` field — compile error
- **Fix:** Added `variant: CardVariant::Bordered` to the struct literal; added `CardVariant` to the use import
- **Files modified:** `ferro-json-ui/src/projection/builder.rs`
- **Commit:** aece2ca1

## Security (T-164-05-01 audit)

Confirmed: `CardVariant` deserialization via serde rejects unknown variant strings (e.g. `"gradient"`) with a clear error at spec-parse time. No class-string injection vector — the variant is a closed enum, not a free-form string that flows into HTML class attributes.

## Notes for Plan 10 (docs)

Plan 10's documentation pass should add a `CardVariant` subsection to `docs/src/json-ui/components.md` under the Card component entry. The section should document:
- The two variants with their CSS output
- When to use Elevated vs Bordered (auth/error vs dashboard)
- The default-preserving behavior (`#[serde(default)]`)
- Example JSON: `{ "type": "Card", "props": { "title": "Login", "variant": "elevated" } }`

## Self-Check: PASSED

- `ferro-json-ui/src/component.rs` — CardVariant enum, CardProps.variant field, 6 tests: FOUND
- `ferro-json-ui/src/render/containers.rs` — CardVariant import, match branch, 3 render tests: FOUND
- `ferro-json-ui/src/lib.rs` — CardVariant re-export: FOUND
- `ferro-json-ui/src/projection/builder.rs` — variant field + import: FOUND
- Commit `46fe1f5c`: component.rs + lib.rs (Task 1)
- Commit `361a6977`: containers.rs (Task 2)
- Commit `aece2ca1`: builder.rs + fmt fixes (Task 3)
- `cargo fmt --all -- --check`: PASSED
- `cargo clippy --all --all-targets -- -D warnings`: PASSED
- `cargo test --all-features`: PASSED (0 failures across all crates)
