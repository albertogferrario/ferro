---
phase: 256
slug: component-renderers-builtin-lockstep
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-06
---

# Phase 256 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec author props → rendered HTML | Prop values (name, price, labels, image_url, stock_badge, form_id, currency) interpolated into HTML/attributes at SSR time | Untrusted spec-authored strings |
| tile/panel/numpad controls → hidden inputs (browser) | Field names used to build `querySelector` attribute selectors in runtime JS | Untrusted field-name strings |
| client-computed running total → operator display | `data-selection-total` is a client-side view; the confirm POST carries only hidden-input quantities | Display-only derived value (never trusted server-side) |
| catalog ↔ ferro-mcp mirror | BUILTIN count/name lockstep across two crates | Introspection metadata integrity |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-256-01 | Tampering | render_tile prop interpolation | mitigate | `html_escape` on name/price/field (atoms.rs:1372–1374), image_url (:1415), stock_badge (:1427); numeric price_cents/qty formatted directly | closed |
| T-256-02 | Tampering | color accent class | mitigate | `Option<Tone>` exhaustive match to full-literal border classes (atoms.rs:1403–1410); no dynamic class construction | closed |
| T-256-03 | Tampering | initQtyButton field selector | accept | Field sanitization `replace(/["\\\]]/g,'')` (tiles.rs:21) present and untouched; display-null relaxation adds no injection surface | closed |
| T-256-04 | Tampering | render_grid row_weights | mitigate | `u8` weights formatted `{w}fr` (containers.rs:877–884) — numeric only, no user string in style attribute | closed |
| T-256-05 | Tampering | tile-grid/filter-tabs interpolation | mitigate | `html_escape` on all_label, item labels, tokens with space→hyphen normalization (atoms.rs:1479–1490) | closed |
| T-256-06 | Tampering | column class construction | mitigate | Exhaustive full-literal match `grid-cols-{1..4}` (containers.rs:953–959); never `format!` | closed |
| T-256-07 | Information disclosure | catalog count/mirror drift | mitigate | Both count guards at 52 (catalog.rs:1252; json_ui_catalog.rs:405); History audit comment (catalog.rs:1248–1251) | closed |
| T-256-08 | Tampering | stepper/numpad/panel prop interpolation | mitigate | `html_escape` on field (atoms.rs:1593), target_field (:1654), form_id/empty_message/currency/total_label (containers.rs:1554–1567) | closed |
| T-256-09 | Tampering | Numpad key values | accept | Keys are a static server-authored `[(&str,&str);12]` array (atoms.rs:1668–1681); no user input reaches `data-numpad-key` | closed |
| T-256-10 | Spoofing/Tampering | client-computed running total (render) | mitigate | Rustdoc on `render_selection_panel` (containers.rs:1534–1536): total display-only, server re-validates qty × price from hidden inputs on POST | closed |
| T-256-11 | Information disclosure | catalog drift at final 52 | mitigate | Count pins + `BUILTIN_SPECS.len()==BUILTIN_TYPES.len()` (catalog.rs:1452) + `component_rule_mapping_is_exhaustive` (json_ui_catalog.rs:756–768) | closed |
| T-256-12 | Tampering | selection.rs querySelector construction | mitigate | `field.replace(/["\\\]]/g,'')` at all 4 selector-build sites (selection.rs:76, :84, :92, :117) | closed |
| T-256-13 | Tampering | line name/qty DOM writes | mitigate | `textContent` only (selection.rs:146, :156, :157, :167); zero `innerHTML` in the file | closed |
| T-256-14 | Spoofing | client running total (runtime) | mitigate | Display-only stated in file header (selection.rs:1–4) and inline (:166); server never receives a total field | closed |
| T-256-15 | Tampering | integer-cents money math | mitigate | `parseInt(...,10) \|\| 0` throughout (selection.rs:79, :87, :118, :125); only float is presentational `(n/100).toFixed(2)` (:180) | closed |
| T-256-16 | Tampering | missing/dynamic CSS class silent break | mitigate | ferro-base.css regen contains every Phase 256 class literal (border-success/warning/destructive, aspect-square, object-cover, overscroll-contain, grid-cols-1..4); no `format!`-built classes | closed |
| T-256-17 | Repudiation | committing unrelated schema churn | accept | D-30: `git status docs/protocol/schemas/` empty after full test run (256-05-SUMMARY.md) — no churn committed | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-256-01 | T-256-03 | Existing tiles.rs field sanitization covers the selector surface; the display-null relaxation only removes an early-return, adding no injection path | plan 256-01 threat model (operator auto chain) | 2026-07-06 |
| AR-256-02 | T-256-09 | Numpad keys are a fixed server-authored array; no user input can reach `data-numpad-key`; the writable target field is html_escaped and runtime-sanitized | plan 256-03 threat model (operator auto chain) | 2026-07-06 |
| AR-256-03 | T-256-17 | Schema-churn discard verified empty by evidence (no real schema change from ferro-json-ui props); phase commits contain only Phase 256 substance | plan 256-05 threat model, D-30 resolution | 2026-07-06 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-06 | 17 | 17 | 0 | gsd-security-auditor (sonnet), post REVIEW-FIX pass |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-06
