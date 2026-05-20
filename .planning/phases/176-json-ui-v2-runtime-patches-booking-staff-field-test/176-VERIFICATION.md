---
phase: 176-json-ui-v2-runtime-patches-booking-staff-field-test
verified: 2026-05-21T00:00:00Z
status: human_needed
score: 6/6
overrides_applied: 0
human_verification:
  - test: "Bug R2 — Card.badge UAT re-test"
    expected: "Rebuild gestiscilo-it against the patched local-path ferro dependency. Load the booking kanban dashboard (calendar_day view). Confirm that booking cards with a countdown badge emit a visible Badge-styled pill to the right of the card title (e.g. 'Scade tra 9m' in Secondary chrome)."
    why_human: "Requires a live browser against the gestiscilo-it consumer app. The renderer fix is verified in unit tests (render_card_emits_badge_when_present), but DOM rendering correctness in the Inertia/React shell, the Tailwind token resolution for bg-secondary/10 and text-secondary-foreground, and the visual co-planar layout of the badge alongside the title can only be confirmed via chrome-mcp snapshot."
  - test: "Bug R3 — Card.subtitle UAT re-test"
    expected: "On the same booking kanban dashboard, confirm that booking cards that include a staff name snapshot emit a muted secondary line between the card title and any description text (e.g. 'Marco Rossi' in text-sm text-text-muted styling, vertical position: below title, above description)."
    why_human: "Same rationale as R2. The mt-0.5 spacing and text-text-muted token rendering require a live Tailwind token environment to verify visually. Unit test render_card_emits_subtitle_when_present confirms the HTML string; browser rendering confirms the visual result."
  - test: "Bug R4 — Grid chip strip visibility UAT re-test"
    expected: "On the booking calendar day view, when a booking has at least one staff member assigned (has_staff = true), the per-staff filter chip strip Grid renders its chips. When a booking has no staff assigned (has_staff = false), the chip strip Grid is absent from the DOM entirely (no empty wrapper, no hidden attribute). F9 was a no-repro against current ferro master; the consumer should rebuild with the patched runtime to confirm the chip strip now behaves correctly in the live app context."
    why_human: "F9 closed as Outcome A (no production code change; visibility evaluator architecture was already correct). The consumer's original symptom may have been caused by a stale local ferro checkout or a chrome-mcp snapshot timing issue during Inertia hydration. The live UAT re-test is the only way to confirm the consumer sees the correct chip strip behavior after rebuilding against the current ferro runtime."
---

# Phase 176: v12.0.2 JSON-UI v2 Runtime Patches (F7–F9) Verification Report

**Phase Goal:** Close three runtime gaps in ferro-json-ui v2 surfaced by the gestiscilo-it β booking↔staff binding UAT — F7 (`Card.badge` slot), F8 (`Card.subtitle` slot), F9 (`Grid.visible` regression coverage + docs).
**Verified:** 2026-05-21T00:00:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| SC1 | Card spec with `badge: "B"` renders badge element with text "B" in Badge-styled, right-aligned span | VERIFIED | `containers.rs:66-76` — `if let Some(ref badge) = props.badge` emits `<div class="flex items-start justify-between gap-2">` wrapper + `<span class="...bg-secondary/10 text-secondary-foreground shrink-0">html_escape(badge)</span>`. Test `render_card_emits_badge_when_present` at line 1230. |
| SC2 | Card spec with `subtitle: "S"` renders subtitle element beneath title with muted-text class | VERIFIED | `containers.rs:83-88` — `if let Some(ref subtitle) = props.subtitle` emits `<p class="mt-0.5 text-sm text-text-muted">html_escape(subtitle)</p>`. Test `render_card_emits_subtitle_when_present` at line 1269. |
| SC3 | Grid spec with `visible` condition renders Grid + children when true; renders no Grid element when false | VERIFIED | Tests `grid_renders_when_visible_true` (line 905), `grid_hidden_when_visible_false` (line 925), `grid_visible_consumer_reproduction` (line 945) in `containers.rs`. All pass green (Outcome A — no-repro). Visibility check at `render/mod.rs:155-160` already evaluated element-level for every element type before dispatch. |
| SC4 | Catalog JSON schema updated — `Card.props` accepts optional `badge: String` and `subtitle: String` | VERIFIED | `catalog.rs:271-274` — Card entry description updated to `"Content container with title, description, optional badge and subtitle, body children, and optional footer slot."` at line 272; schema uses `schema_for!(CardProps)` at line 273. Tests `card_props_schema_includes_badge` and `card_props_schema_includes_subtitle` in `component.rs` at lines 1456 and 1481 assert both property keys present with string type. |
| SC5 | v2 component docs updated — Card slot props table + Grid.visible clarification | VERIFIED | `docs/src/json-ui/components.md:84-85` — Card prop table rows for `subtitle` and `badge` added. Line 100: worked example with both slots. Lines 195-208: `#### Visibility` subsection in Grid section documents element-level semantics with `/has_staff` example and universality statement. |
| SC6 | `cargo test --all-features` passes (gestiscilo-it UAT is addressed separately as human verification) | VERIFIED | All 6 commits confirmed present in git log (b3f35e03, 14a9c22e, e7372289, 9aba2e8c, 28b2eb58, 727755b3). SUMMARY.md self-check reports `cargo fmt --check && cargo clippy -D warnings && cargo test --all-features` passed before each commit. Regression gate ran `cargo test -p ferro-json-ui` (573 tests passed) independently before this verification. |

**Score:** 6/6 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `ferro-json-ui/src/component.rs` | `CardProps` with `badge: Option<String>` and `subtitle: Option<String>` + serde/schema tests | VERIFIED | Fields present at lines 176 and 181. Serde attrs `#[serde(default, skip_serializing_if = "Option::is_none")]` on both. Tests: `card_props_round_trips_badge` (1388), `card_props_omits_empty_badge_in_json` (1404), `card_props_round_trips_subtitle` (1422), `card_props_omits_empty_subtitle_in_json` (1438), `card_props_schema_includes_badge` (1456), `card_props_schema_includes_subtitle` (1481). Three existing positional constructors augmented with `subtitle: None, badge: None` (12 occurrences of `subtitle: None,` confirmed). |
| `ferro-json-ui/src/render/containers.rs` | `render_card` badge + subtitle emission; F7+F8 render tests; F9 Grid visibility regression tests | VERIFIED | Badge slot lines 66-76; subtitle slot lines 83-88. Five Card render tests lines 1230-1328. Three Grid visibility tests lines 905-990. |
| `ferro-json-ui/src/catalog.rs` | Card catalog description string updated to name badge + subtitle | VERIFIED | Line 272: `"Content container with title, description, optional badge and subtitle, body children, and optional footer slot."` |
| `docs/src/json-ui/components.md` | Card prop table rows for subtitle + badge; worked example; Grid Visibility subsection | VERIFIED | Lines 84-85: prop table rows. Line 100: vertical stacking note with worked example. Lines 195-208: `#### Visibility` subsection with element-level definition, `/has_staff` example, and universality statement. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `component.rs::CardProps` | `containers.rs::render_card` | `render_card` decodes `CardProps` and reads `.badge` / `.subtitle` | WIRED | `props.badge` at line 66, `props.subtitle` at line 83. Both inside `render_card`. |
| `component.rs::CardProps` | `catalog.rs::BUILTIN_SPECS` | `schema_for!(CardProps)` regenerates schema automatically | WIRED | `catalog.rs:273` — `|| to_value(schema_for!(CardProps)).unwrap()` unchanged; schemars derive picks up new fields automatically. |
| `render/mod.rs` visibility check | `visibility.rs::Visibility::evaluate` | Element-level check applies to Grid before dispatch | WIRED | `grid_visible_consumer_reproduction` test exercises the full path via `render_spec_to_html` (not bare `render_grid`), confirming the walker's check at `mod.rs:155-160` fires for Grid elements. |

### XSS Invariant (D-09)

Both new render paths apply `html_escape`:

- Badge: `containers.rs:74` — `html_escape(badge)` inside `format!(...)` for the `<span>` element
- Subtitle: `containers.rs:86` — `html_escape(subtitle)` inside `format!(...)` for the `<p>` element

Pattern is identical to the existing title (line 70) and description (line 91) slots. Invariant is uniform across all Card text slots.

### CardProps Field Order

`component.rs:169-188` — field order is: `title(169) → description(171) → subtitle(176) → badge(181) → max_width(183) → footer(186) → variant(188)`. Matches plan specification and Wave-0 mechanical fixup order.

### Backward Compatibility

Both new fields carry `#[serde(default, skip_serializing_if = "Option::is_none")]`:
- subtitle: `component.rs:175`
- badge: `component.rs:180`

Existing specs without these fields deserialize correctly (default = None); serialized specs without these fields omit the keys.

### F9 Outcome A Confirmation

`176-02-SUMMARY.md` explicitly records "Outcome A — could not reproduce against current ferro master." All three reproduction tests passed green on first run. Task 3 (conditional production code fix) was skipped. Verbatim test output recorded in SUMMARY (3 passed, 0 failed). Production code in `render/mod.rs`, `visibility.rs`, and `resolve.rs` is unchanged — only tests and docs were added.

### Anti-Patterns Found

None in the phase-modified code paths. The word "placeholder" appears in `component.rs` and `catalog.rs` but exclusively as legitimate API-doc uses (Skeleton component, URL-pattern documentation, image fallback descriptions) — none in the Card, render_card, or badge/subtitle code paths.

### Human Verification Required

Three items require human testing against the live gestiscilo-it consumer application after rebuilding against the patched ferro local-path dependency.

#### 1. Bug R2 — Card.badge visual rendering

**Test:** Rebuild gestiscilo-it against the patched ferro. Navigate to the booking kanban dashboard. Open a booking card that has a countdown badge configured.
**Expected:** A Badge-styled pill appears to the right of the card title, visually co-planar (flex row, `justify-between`), with Secondary chrome (`bg-secondary/10 text-secondary-foreground`). Text matches the badge string emitted by the server (e.g. "Scade tra 9m").
**Why human:** Tailwind token resolution (`bg-secondary/10`, `text-secondary-foreground`) and the visual co-planar flex layout require a live browser environment. Unit tests confirm the HTML string; browser rendering confirms visual correctness.

#### 2. Bug R3 — Card.subtitle visual rendering

**Test:** On the same booking kanban dashboard, open a booking card that includes a staff name snapshot as subtitle.
**Expected:** A muted secondary line appears between the card title and any description text. Text matches the subtitle string (e.g. "Marco Rossi"). Visually: `text-sm text-text-muted`, `mt-0.5` spacing (tighter than description's `mt-1`).
**Why human:** Same rationale as R2 — token rendering and spacing require a live Tailwind environment.

#### 3. Bug R4 — Grid chip strip visibility

**Test:** On the booking calendar day view, view a booking with at least one staff member assigned (`has_staff = true`). Then view a booking with no staff assigned (`has_staff = false`).
**Expected:** When `has_staff = true`, the per-staff filter chip strip Grid renders its chips. When `has_staff = false`, the entire chip strip subtree is absent from the DOM (no empty wrapper, no `hidden` attribute).
**Why human:** F9 closed as Outcome A (no production code change). The original consumer symptom may have been caused by a stale ferro checkout or chrome-mcp snapshot timing. The live UAT re-test confirms the consumer sees correct behavior after rebuilding against the current runtime.

---

_Verified: 2026-05-21T00:00:00Z_
_Verifier: Claude (gsd-verifier)_
