---
phase: 257
slug: projection-builder-register-layout-template
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-06
---

# Phase 257 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec author → Catalog::validate | Untyped spec JSON crosses into the validation gate before render. This phase widens the validation skip surface for `$each` template elements. | Spec element props (data-bound JSON values) |
| ServiceDef fields/actions → projection-emitted spec → HTML render | Field values become `$data` bindings; resolved + escaped by the existing render pipeline. | Field names (author-controlled), prop pointers (app-controlled) |
| client → /cassa (GET), /cassa/conferma (POST) | Public demo routes; no auth in milestone scope. | Demo product rows, form submissions |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-257-01 | Tampering (validation bypass) | Catalog::validate Stage 2/3 `each.is_some()` guard | mitigate | Guard strictly scoped to `each.is_some()` (catalog.rs:770 Stage 2, catalog.rs:861 Stage 3). Non-template elements retain full per-element + envelope schema validation. `validate_directives` in spec.rs:843 still enforces `$each` structural rules (EachPathNotArray, reserved `as` names, nested-$each rejection). `resolve_expressions` enforces concrete prop types at render time. | closed |
| T-257-02 | Information Disclosure | ElementBuilder.each / fill_viewport setters | accept | Pure in-process builder methods over already-existing private fields. No I/O, no auth, no serialization of secrets. No realistic threat. See Accepted Risks Log. | closed |
| T-257-03 | Information Disclosure | emit_register_root meaning-driven Tile mapping | mitigate | Tile props mapped only from `Identifier`/`EntityName`/`Money` meanings via `field_name_by` (builder.rs:618-622). Fallback (post-WR-02 fix, commit 3309a043) uses `f.readable && lookup_meaning(&f.meaning).display.is_some()` (builder.rs:628), which structurally excludes Sensitive/ForeignKey meanings whose `display` is `None` in `lookup_meaning`. Regression test `register_projection_fallback_excludes_sensitive_fields` added. | closed |
| T-257-04 | Tampering (injection) | $data prop bindings emitted by the projector | mitigate | Projector emits `{"$data":"/p/…"}` pointer objects only (builder.rs:699-707). No raw string interpolation of field values into HTML. Resolution and escaping owned by the existing `resolve_expressions`/render pipeline. No `RawHtml` anywhere in cassa.rs (grep-clean). | closed |
| T-257-05 | Denial of Service | Tile `$each` expansion over `/data/{service}` | accept | Iterated array is server-supplied by the handler (app-controlled), not end-user input. No untrusted array-size amplification path in this phase. See Accepted Risks Log. | closed |
| T-257-06 | Elevation of Privilege / Spoofing | /cassa demo routes; deleted rimuovi endpoint | accept | `/cassa` + `/cassa/conferma` are unauthenticated demo endpoints with no state mutation (`conferma` is a plain redirect). The `rimuovi` route deletion reduces attack surface — only `cassa.index` and `cassa.conferma` remain in routes.rs (verified: grep for `rimuovi` in routes.rs returns nothing). See Accepted Risks Log. | closed |
| T-257-07 | Tampering (injection) | product rows → HTML via projection | mitigate | Rows emitted as `$data` bindings resolved + escaped by the existing render pipeline. `grep -rn RawHtml app/src/controllers/cassa.rs` returns nothing. `render_file` absent. The flip from hand-authored JSON to projection derivation removes the last manual spec path. | closed |

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-257-01 | T-257-02 | ElementBuilder.each and SpecBuilder.fill_viewport are pure in-process builder methods that set already-existing private fields. They introduce no I/O, no auth paths, and no serialization of secrets. The spec author is the app author, not an end user. No realistic information disclosure path exists. | Phase 257 plan author | 2026-07-06 |
| AR-257-02 | T-257-05 | The array iterated by Tile `$each` is at `/data/{service.name}`, populated entirely by the handler from server-controlled data (`cassa_products()`). No end-user-supplied input flows into the array size or content at this phase. Unbounded array growth is a handler responsibility, not introduced by this phase. | Phase 257 plan author | 2026-07-06 |
| AR-257-03 | T-257-06 | The `/cassa` GET and `/cassa/conferma` POST routes are explicitly unauthenticated demo endpoints for the POS sample. `conferma` performs a plain redirect with no state mutation. The deletion of `rimuovi` strictly reduces the attack surface compared to the pre-phase state. Authentication is out of scope for this sample (documented in RESEARCH Security Domain). | Phase 257 plan author | 2026-07-06 |

---

## Unregistered Threat Flags

None. All three SUMMARY.md files (Plans 01, 02, 03) explicitly reported "No threat flags" in their Threat Surface Scan sections.

---

## Evidence Index

| Threat ID | Evidence Location | Verified Pattern |
|-----------|-------------------|-----------------|
| T-257-01 Stage 2 | ferro-json-ui/src/catalog.rs:770 | `if el.each.is_some() { continue; }` |
| T-257-01 Stage 3 | ferro-json-ui/src/catalog.rs:861-865 | `obj.remove("props")` for template elements |
| T-257-01 validate_directives | ferro-json-ui/src/spec.rs:843 | `fn validate_directives` enforces EachPathNotArray/reserved-as/nested-each |
| T-257-02 | ferro-json-ui/src/spec.rs:401, 525 | `pub fn fill_viewport`, `pub fn each` — no I/O |
| T-257-03 primary | ferro-json-ui/src/projection/builder.rs:618-622 | `field_name_by` with `f.readable && pred(&f.meaning)` |
| T-257-03 fallback (WR-02) | ferro-json-ui/src/projection/builder.rs:628 | `.find(|f| f.readable && lookup_meaning(&f.meaning).display.is_some())` |
| T-257-04 | ferro-json-ui/src/projection/builder.rs:699-707 | `serde_json::json!({"$data": format!("/p/{id_field}")})` pointer-only emission |
| T-257-06 rimuovi absent | app/src/routes.rs:23-24 | Only `cassa.index` and `cassa.conferma` present; no `rimuovi` |
| T-257-07 RawHtml absent | app/src/controllers/cassa.rs | `grep -n RawHtml` returns nothing |
| T-257-07 register_template | app/src/controllers/cassa.rs:2,74,77 | `register_template` wired; no `render_file` |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-06 | 7 | 7 | 0 | gsd-secure-phase (Claude Sonnet 4.6) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-06
