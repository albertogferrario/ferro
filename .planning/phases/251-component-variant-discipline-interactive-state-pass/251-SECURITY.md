---
phase: 251
slug: component-variant-discipline-interactive-state-pass
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-03
---

# Phase 251 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec JSON → props structs | Agent/app-authored spec values (variant/tone/size, badge row data) cross into typed Rust via serde | Untrusted spec values |
| props → HTML markup | User-supplied strings (labels, badge text) rendered into HTML | Untrusted display strings |
| data attributes → runtime JS | Toast/tab runtime JS reads DOM attributes/classes the SSR emitted and toggles class strings | Attribute values from SSR output |
| spec JSON → catalog validation | Assembled JSON Schema is the enforcement point rejecting non-canonical enum values at spec-load | Untrusted spec documents |
| agent → generated spec | ferro-mcp templates/prose steer what an agent authors | Agent-facing guidance text |
| render-source literals → generated CSS | Tailwind scanner turns source-literal classes into ferro-base.css; dynamically-constructed classes are silently purged | Class-name literals |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-251-01 | Tampering | DataTable BadgeCell / MediaCardGrid tone from row data | mitigate | Row tone decoded through typed `Tone` enum (`data.rs:360-373` BadgeCell, `:686-695` MediaCardGrid, Neutral fallback); invalid values → escaped HTML-comment diagnostic; class strings from exhaustive full-literal match arms (`atoms.rs:274-279`); raw row value never reaches a class attribute | closed |
| T-251-02 | Tampering (XSS) | Toast/tab runtime JS (`data-toast-tone`, class toggling) | mitigate | Fixed `VARIANT_CLASSES` map with `neutral` fallback (`runtime/toasts.rs:7-12,19-21`); innerHTML only via `escapeHtml` (`:26-29,72-76`); dismissal removes node, renders nothing attribute-derived; tabs toggle constant literals only (`runtime/tabs.rs:65-70`); JS↔SSR lockstep pinned by `toast_tone_classes_match_ssr` (`runtime/mod.rs:111-130`) + retired-attribute guard (`:81`) | closed |
| T-251-03 | Tampering (XSS) | User labels in badge/alert/toast/button/actioncard + migrated interactive sites | mitigate | `html_escape` preserved at every touched render site (Button `atoms.rs:172-243`, Alert `:321-324`, Toast `:869`, ActionCard `:1320-1346`, Badge `:288`); wrapper markup server-controlled; escape-call density: atoms 81 / form 80 / containers 75 / data 35 | closed |
| T-251-04 | Elevation of Privilege | `ActionItem.visible_if` gate (touched by `variant`→`Variant` rename) | accept | Rename changed only the field type to closed `Option<Variant>` (`component.rs:987`); fail-closed gate (`containers.rs:986-1010`, `None`/`Null` → hidden) diffed byte-identical against pre-phase commit `0f4fbe94~1`; no new input surface | closed |
| T-251-05 | Denial of Service | Toast dismissal via `transitionend` under reduced motion | mitigate | Reduced motion collapses durations to `0.01ms !important` — not `none` — so `transitionend` fires (`assets/input.css:96-102`, present in generated ferro-base.css); 500ms `setTimeout` fallback + `removed` idempotency guard (`runtime/toasts.rs:59-69`); non-dismissible toasts clamp timeout ≥ 1s (`atoms.rs:851-862`, WR-04) — no stuck toast nodes | closed |
| T-251-06 | Tampering | Out-of-vocabulary enum value in authored spec | mitigate | Closed schemars enums advertise only canonical values (`component.rs:27-59`); `Catalog::validate` Stage 2 rejects unknown values at spec-load (`catalog.rs:711-752`); Stage 2b retired-prop lint with `RETIRED_PROPS` table + recursive confirm/notify walk (`catalog.rs:754-777,883-923`, WR-01); serde rejection pinned for xs/default/link/info/error (`component.rs:1950-1981`); D-19 zero-exclusion $ref-walking drift guard with non-vacuity counter (`catalog.rs:1338-1408`) | closed |
| T-251-07 | Repudiation / correctness | Stale agent-facing templates steering invalid specs | mitigate | ferro-mcp surface fully canonical (`code_templates.rs:1095`, `json_ui_validate_spec.rs:137`, `json_ui_catalog.rs:277-279`); remaining `variant` mentions in catalog prose are legitimate canonical usage — zero retired values | closed |
| T-251-08 | Denial of Service (visual) | Emitted class purged from ferro-base.css | mitigate | Dynamic class-construction grep = 0 hits in `render/`; `classes.rs` constants are complete literals with composition drift guard (`classes.rs:48`); ferro-base.css spot-greps all present: `focus-visible:ring-ring`, `duration-fast/base`, `after:duration-fast`, `disabled:pointer-events-none`, `border-l-success`, `peer-focus:ring-ring`, `focus-visible:ring-inset`, `ease-base` | closed |
| T-251-09 | Tampering | Docs/migration table (public artifact) | accept | Documentation only, no runtime input surface (`docs/src/json-ui/components.md:72-91`); claim text accurately reflects enforcement behavior (retired values fail at parse, retired prop names fail catalog validation) | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| R-251-01 | T-251-04 | The `variant`→`Variant` rename touched only the field's type on `ActionItem`; the fail-closed `visible_if` visibility gate is byte-identical to its pre-phase state and no new input surface was introduced | gsd-security-auditor (verified) / auto-chain | 2026-07-03 |
| R-251-02 | T-251-09 | Migration table and canonical-enum docs are documentation-only artifacts with no runtime input surface; content reviewed for accurate old→new mapping | gsd-security-auditor (verified) / auto-chain | 2026-07-03 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-03 | 9 | 9 | 0 | gsd-security-auditor (sonnet), orchestrated via /gsd-secure-phase 251 |

**Test evidence basis:** crate gate reused per serialize-CPU policy — `cargo test -p ferro-json-ui` 635 passed + crate clippy `--all-targets --all-features -D warnings` clean at final code commit `116447ce`; all later commits are docs/planning-only and the `ferro-json-ui`/`ferro-mcp` trees were clean at audit time.

**Informational notes (not blockers):**
1. Pre-existing unescaped `props.icon` interpolation in ActionCard (`atoms.rs:1339`) — predates phase 251 (byte-identical at `0f4fbe94~1`), so the T-251-03 "escaping preserved" mitigation holds; flagged for a future phase's threat model (icons-as-markup vs icons-as-text consistency decision).
2. ~200 bytes of dead retired utilities (`duration-150/300`, `ring-primary`) leak into ferro-base.css from negative test assertions; unreferenced by any render output, no security effect; Phase 252 lint candidate.

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-03
