---
phase: 251-component-variant-discipline-interactive-state-pass
verified: 2026-07-03T16:05:00Z
status: human_needed
score: 10/11 must-haves verified
overrides_applied: 0
human_verification:
  - test: "Open the sample app auth pages (/auth/login, /auth/login/confirm) in a browser, in BOTH light and dark mode. Tab through the page; hover buttons/links; trigger the 422 error path (bad email)."
    expected: "focus-visible ring (--color-ring) visibly appears on keyboard focus for every interactive element; hover treatments render; transitions are smooth with no pop/reflow; disabled controls are non-interactive; the intended visual deltas are present (Badge neutral = outlined, Alert neutral = surface tint not primary tint, ActionCard neutral = plain left border, relationship buttons = ghost not underline-link); dark-mode contrast is acceptable."
    why_human: "Plan 04 Task 3 was a checkpoint:human-verify that was auto-approved under the --auto chain with HTML/CSS class-level evidence only (no Chrome MCP available in that session). Class presence in served markup is verified programmatically, but pixel-level rendering quality (ring visibility, hover smoothness, dark contrast, no reflow) cannot be. The 04-SUMMARY itself flags this pass as still worthwhile; it can be folded into the Phase 253 pre-publish review."
---

# Phase 251: Component Variant Discipline + Interactive-State Pass Verification Report

**Phase Goal:** One variant vocabulary across the whole component set — audit all 47 builtin components, normalize to canonical `variant` (primary/secondary/outline/ghost/destructive), `tone` (neutral/success/warning/destructive), and `size` (sm/md/lg) enums, and bring every interactive component to the quality bar: hover, `focus-visible` ring, disabled treatment, frequency-tiered motion.
**Verified:** 2026-07-03T16:05:00Z
**Status:** human_needed
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| #   | Truth | Status | Evidence |
| --- | ----- | ------ | -------- |
| 1 | SC1: Every component exposing weight/status/size props uses the canonical enums; catalog prop schemas enforce them; drift guards extended to the enum sets | ✓ VERIFIED | `pub enum Variant/Tone/Size/CardAppearance` at component.rs:27/53/77/203 with exactly the canonical value sets; 11 typed `pub (variant\|tone\|size\|appearance): Variant/Tone/Size/CardAppearance` serde fields; zero retired enum identifiers in code (4 remaining hits are historical comments only); `variant_tone_size_enum_sets_drift_guard` (catalog.rs:1339) runs and passes — $ref-resolving walker with visited-set (:1289-1331) and non-vacuity `checked >= 10` (:1404); WR-01 added a `RETIRED_PROPS` catalog lint (:762,:883) rejecting renamed prop names |
| 2 | SC2: Every interactive component has hover, `focus-visible` (via `--color-ring`), and disabled states; transitions use motion tokens at frequency-appropriate tiers | ✓ VERIFIED | classes.rs defines FOCUS_RING (`focus-visible:ring-ring`), MOTION_FAST/BASE, DISABLED_BASE, INTERACTIVE_BASE; 68 consumption sites across atoms/containers/data/form/layout; zero `duration-150`/`duration-300`/`focus-visible:ring-primary`/`motion-reduce:transition-none` in non-test render/layout/runtime code (only negative test assertions remain); WR-03 filled 6 straggler sites incl. `after:duration-fast`/`after:ease-base` Switch knob |
| 3 | SC3: A migration table lists every renamed prop/value for consumers | ✓ VERIFIED | `## Component vocabulary migration` at docs/src/json-ui/components.md:72; 14 `variant` rows; key rows present (`link→ghost`, `xs→sm`, Card `variant→appearance`, `badge_variant_key→badge_tone_key`, ConfirmDialog/Notify `variant→tone`) plus visual-delta notes |
| 4 | SC4: `ferro-base.css` regenerated after class changes; workspace gate green | ✓ VERIFIED | Last regen commit 76529cb1 (WR-03, the final class-changing commit) — `after\:duration-fast`, `after\:ease-base`, `backdrop-blur-md`, `bg-success\/70`, `focus-visible\:ring-ring`, `disabled\:pointer-events-none`, `border-l-success`, `ring-inset` all present; full CI-exact gate documented green in 04-SUMMARY (fmt/clippy --all-features/test --all-features exit 0); fresh evidence: `cargo test -p ferro-json-ui` 635 passed, 0 failed |
| 5 | A spec using a retired value (`size: xs`, `variant: link`, `tone: info`) fails serde/catalog parse instead of silently rendering | ✓ VERIFIED | Negative tests `retired_size_values_are_rejected` (:1950), `retired_variant_values_are_rejected` (:1962), `retired_tone_values_are_rejected` (:1974) in component.rs — all in the passing 635 |
| 6 | The projection builder emits canonical values (Ghost relationship buttons, Neutral badges, Bordered cards) | ✓ VERIFIED | `Variant::Ghost` at component_map.rs:346, `Tone::Neutral` return at :170, `CardAppearance::Bordered` at builder.rs:370 |
| 7 | Toast SSR markup and the toast runtime JS agree on tone attribute + classes (JS-SSR lockstep) | ✓ VERIFIED | `data-toast-tone` in both atoms.rs (2) and runtime/toasts.rs (1); zero `data-toast-variant` outside negative assertions; WR-02 `TOAST_TONE_*` consts in classes.rs shared by both sides; `toast_tone_classes_match_ssr` test passes; `transitionend` + 500ms fallback dismissal (toasts.rs:57-67); WR-04 `[data-toast-close]` wired SSR→JS (atoms.rs:769-785, toasts.rs:92-102) |
| 8 | The interactive base string is defined once (shared constants), not hand-copied across 47 sites | ✓ VERIFIED | Single definition in render/classes.rs; composition drift-guard test at classes.rs:54; 68 composition sites reference the constants |
| 9 | Agent-facing ferro-mcp text uses the canonical vocabulary — no stale `variant: default`/`info`/`DialogVariant` | ✓ VERIFIED | Zero retired-vocabulary hits in ferro-mcp/src/tools; `"variant": "primary"` in code_templates.rs:1095; ACTION_API prose shows `tone: Tone (neutral\|success\|warning\|destructive)` and `tone: Tone (neutral\|destructive)` (json_ui_catalog.rs:277-279) |
| 10 | Docs "Shared Enum Values" section describes exactly three enums with canonical value sets | ✓ VERIFIED | components.md section lists exactly **variant**/**tone**/**size** with canonical values + "one word, one meaning" statement; component-scoped enums split into "Component-Specific Enum Values"; zero stale `button_variant`/`badge_variant`/`alert_variant`/`toast_variant`/`card_variant` in docs/src/json-ui outside the migration table |
| 11 | The sample app renders correctly in light and dark with the intended visual deltas and no unstyled/broken interactive states | ? UNCERTAIN | Class-level evidence recorded in 04-SUMMARY (served pages carry canonical classes, zero retired classes, both theme blocks define `--color-ring`, served CSS byte-identical to regen) — but the pixel-level pass was auto-approved without a human eye. Routed to human verification |

**Score:** 10/11 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
| -------- | -------- | ------ | ------- |
| `ferro-json-ui/src/component.rs` | Shared canonical enums, per-component enums deleted | ✓ VERIFIED | 4 canonical enums with exact value sets; retired identifiers appear only in comments |
| `ferro-json-ui/src/action.rs` | ConfirmDialog/Notify normalized to shared `Tone` (OQ-1) | ✓ VERIFIED | `tone` fields on shared Tone; Notify success default preserved via serde default fn |
| `framework/src/lib.rs` | Facade re-exports canonical enums, old names removed | ✓ VERIFIED | `CardAppearance`, `Size`, `Tone`, `Variant` in json-ui re-export block (:88-97); zero retired names |
| `ferro-json-ui/src/render/classes.rs` | Shared interactive-base constants | ✓ VERIFIED | FOCUS_RING/MOTION_FAST/MOTION_BASE/DISABLED_BASE/INTERACTIVE_BASE + WR-02 TOAST_TONE_* consts; composition drift-guard test |
| `ferro-json-ui/src/layout.rs` | Sidebar nav composed from shared constants; INT-07 flipped | ✓ VERIFIED | INTERACTIVE_BASE composition; `ring-ring`/`duration-fast` assertions in tests |
| `ferro-json-ui/src/catalog.rs` | D-19 schema-walking drift guard + canonical prose | ✓ VERIFIED | `variant_tone_size_enum_sets_drift_guard` passes standalone; RETIRED_PROPS lint (WR-01); prose canonicalized |
| `ferro-mcp/src/tools/code_templates.rs` | Spec template emits canonical values | ✓ VERIFIED | `"variant": "primary"` at :1095; zero retired values |
| `docs/src/json-ui/components.md` | Canonical enum section + D-17 migration table | ✓ VERIFIED | Migration table at :72; three-enum section; WR-05 prop-doc corrections in tree |
| `ferro-json-ui/assets/ferro-base.css` | Regenerated stylesheet covering all emitted classes | ✓ VERIFIED | All 8 smoke-check classes present (escaped selectors); regenerated in the last class-changing commit (76529cb1) |

### Key Link Verification

| From | To | Via | Status | Details |
| ---- | --- | --- | ------ | ------- |
| component.rs props structs | shared Variant/Tone/Size enums | typed serde fields | ✓ WIRED | 11 typed fields incl. `appearance: CardAppearance`, `variant: Option<Variant>` (ActionItem), StatCard `tone: Tone` |
| atoms.rs (Toast) | runtime/toasts.rs | `data-toast-tone` + TOAST_TONE_* consts | ✓ WIRED | Attribute in both files; shared consts asserted verbatim by `toast_tone_classes_match_ssr` |
| render/*.rs + layout.rs sites | render/classes.rs constants | const composition | ✓ WIRED | 68 references across 5 consumer files; zero remaining inline retired fragments |
| containers.rs (tabs) | runtime/tabs.rs | classList literals | ✓ WIRED | Tab literals unchanged by the class pass (02-SUMMARY grep-verified); no drift introduced |
| catalog full_schema | canonical enum value sets | $ref-resolving walker | ✓ WIRED | Guard walks oneOf props subtrees AND all root $defs with visited-set + non-vacuity counter; passes |
| ferro-mcp catalog/templates | canonical vocabulary | auto-derived schemas + prose | ✓ WIRED | Zero retired mentions; prompt inlines canonical enum values (`prompt_inlines_canonical_enum_values`) |
| input.css safelist + source literals | ferro-base.css | gen-ferro-base-css.sh scan | ✓ WIRED | All new literals surfaced in regen output; dynamic-construction grep 0 hits |
| Plan 01/02 SUMMARY deltas | components.md migration table | verbatim old→new rows | ✓ WIRED | All migration_table_content rows present incl. visual-delta notes and OQ-3 dot_colors caveat |

### Data-Flow Trace (Level 4)

Not applicable in the app-data sense (Rust rendering library, no dynamic data fetch). The analogous chain — enum value → render match arm → class literal → generated CSS — was traced end-to-end: canonical enum values drive exhaustive match arms emitting full class literals (no dynamic construction, grep 0 hits), and every emitted literal is present in the regenerated `ferro-base.css`.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
| -------- | ------- | ------ | ------ |
| Crate suite green post-review-fixes | `cargo test -p ferro-json-ui` | 635 passed; 0 failed (matches REVIEW-FIX claim) | ✓ PASS |
| D-19 drift guard runs and passes | `cargo test -p ferro-json-ui variant_tone_size_enum_sets_drift_guard` | 1 passed | ✓ PASS |
| Toast JS/SSR lockstep test passes | `cargo test -p ferro-json-ui toast_tone_classes_match_ssr` | 1 passed | ✓ PASS |
| ferro-mcp json_ui suite | not re-run | ferro-mcp untouched since its last green run (Plan 03: 303+; Plan 04 gate: 47/47) — evidence reused per CPU-serialization policy | ? SKIP |
| Full `--all-features` workspace gate | not re-run | explicitly prohibited by verification instructions; Plan 04 documented all three CI-exact commands exit 0 | ? SKIP |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
| ----------- | ----------- | ----------- | ------ | -------- |
| DS-03 | 251-01, 251-03, 251-04 | 47 components on canonical variant/tone/size; schemas enforce; drift guards extended; migration table | ✓ SATISFIED | Truths 1, 3, 5, 6, 9, 10 |
| DS-04 | 251-02, 251-04 | Hover/focus-visible/disabled on every interactive component; motion tokens; ferro-base.css regen | ✓ SATISFIED | Truths 2, 4, 7, 8 (truth 11 pixel-level confirmation pending human) |

No orphaned requirements: REQUIREMENTS.md maps exactly DS-03 and DS-04 to Phase 251; both are claimed across the four plans. Note (info): the REQUIREMENTS.md traceability table rows for DS-03/DS-04 still read "Not started" while the requirement checkboxes are `[x]` — bookkeeping drift, normally reconciled at milestone completion.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
| ---- | ---- | ------- | -------- | ------ |
| — | — | No TODO/FIXME/placeholder/stub patterns in any modified file | — | — |

Disconfirmation-pass observations (all ℹ️ Info, none blocking):

1. **Element-level typed `action` fields escape the WR-01 retired-prop lint** — the lint's recursive walk covers props-embedded actions (row_actions, buttons) but element-level `action` is deserialized before validate sees it. Documented limitation in REVIEW-FIX; stale-prop detection is handed off to Phase 252's design lint.
2. **`alert_emits_message_and_role` still sends retired `variant` to Alert and passes** (IN-03) — render-time serde decode is intentionally lenient; enforcement lives in `Catalog::validate`. Out of fix scope by review policy.
3. **Retired utility definitions leak into ferro-base.css from negative test assertions** (~200 bytes dead CSS: `duration-150/300`, `ring-primary` family) — documented in 04-SUMMARY, unreferenced by any markup, Phase 252 cleanup candidate.

### Human Verification Required

### 1. Pixel-level light + dark visual pass on the sample app

**Test:** Open `/auth/login` and `/auth/login/confirm` in a browser in both light and dark mode. Tab through every interactive element; hover buttons/links/nav; submit a bad email to exercise the 422 error path; if a toast can be triggered, verify fade + close button.
**Expected:** focus-visible ring appears on keyboard focus (from `--color-ring`); hover treatments render; no unstyled elements; no pop/reflow on transitions; intended deltas present (Badge neutral outlined, Alert neutral surface-tinted, ActionCard neutral plain border, ghost relationship buttons); error input shows `ring-destructive`; dark-mode contrast acceptable.
**Why human:** Plan 04's blocking `checkpoint:human-verify` was auto-approved under the `--auto` chain using served-HTML class evidence (Chrome MCP unavailable in the executor session). All class-level checks passed, but visual rendering quality is not programmatically verifiable. The 04-SUMMARY itself recommends this pass at the Phase 253 pre-publish review — doing it there satisfies this item.

### Gaps Summary

No gaps. All four ROADMAP success criteria are verified in the current tree, which includes the five review-fix commits (e1b5c520, e74d2ba6, 76529cb1, 116447ce, 34938db2) — notably WR-03's straggler class pass regenerated `ferro-base.css` again after the Plan 04 regen, keeping SC4's "regenerated after class changes" true for the final tree state. All 17 documented commits verified in git. Fresh crate-scoped test evidence (635 passed) confirms the review-fix claim. The single open item is the pixel-level visual pass that the auto-chain could not perform; it is a confirmation of an already class-level-verified state, and can be folded into the Phase 253 pre-publish review.

---

_Verified: 2026-07-03T16:05:00Z_
_Verifier: Claude (gsd-verifier)_
