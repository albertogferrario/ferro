---
phase: 254
slug: props-contracts-touch-foundation-design-rules
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-05
---

# Phase 254 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec JSON → typed `*Props` | Developer/agent-authored JSON deserialized into typed structs at decode time; no runtime end-user input this phase | Component props (public contract, non-sensitive) |
| `ProductTileProps.categories` → HTML attribute | Category strings (developer- or data-sourced) written into `data-product-categories` at render time — the one boundary where untrusted text reaches emitted markup this phase | Category labels (untrusted text) |
| crate source literals → Tailwind CSS generation | The `@source` scanner reads Rust string literals to emit CSS; a dynamic class would silently not generate | Class-name literals (non-sensitive) |
| developer/agent-authored spec file → `design::lint` | Lint reads raw `Spec` JSON at dev/CI time, emits diagnostics only; never renders or executes spec content | Spec structure (non-sensitive) |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-254-01 | Tampering | `ProductTileProps` / new `*Props` deserialization | mitigate | serde strict typing rejects malformed shapes at decode; ill-typed props surface as `decode_diagnostic("ProductTile", …)` (`render/atoms.rs:1366`), never silent execution | closed |
| T-254-02 | Information disclosure | schema smoke tests / JsonSchema derive | accept | Schemas describe public component contracts intended for MCP/doc introspection; no secrets | closed |
| T-254-03 | Tampering (HTML/attribute injection, XSS) | `render_product_tile` `data-product-categories` emission | mitigate | `html_escape(…)` wraps the space-normalized token join (`render/atoms.rs:1380-1390`); asserted by `product_tile_escapes_categories` (atoms.rs:2611) and `product_tile_normalizes_spaces_in_category_names` (atoms.rs:2599); escaping verified intact after the WR-01 normalization fix (f91f6a40) | closed |
| T-254-04 | Denial of service (silent CSS non-generation) | POS constants / `pos-tap-highlight` utility | mitigate | Every constant is a complete class literal (drift-guard `pos_render_functions_use_constants_not_literals`); `pos-tap-highlight` is a real `@utility` (`assets/input.css:103`); generated `ferro-base.css` verified to contain `pos-tap-highlight`, `overscroll-contain`, `active:scale-95` | closed |
| T-254-05 | Repudiation / Information disclosure | `render::classes` module made `pub` | accept | Exposes only class-string constants and their module path; no logic, no secrets; pre-1.0 API surface changes acceptable | closed |
| T-254-06 | Tampering | `design::lint` rule checks | accept | Rules are pure functions over the parsed Spec, diagnostics-only (252 D-12); a malicious spec can at most produce misleading findings, never code execution | closed |
| T-254-07 | Information disclosure | patterns.md / RULE_COMPONENTS | accept | Public design-system documentation and a component-name mapping; no secrets | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-254-01 | T-254-02 | JsonSchema output is the intended public introspection surface (MCP catalog / docs); nothing sensitive is derivable from it | gsd-secure-phase (plan-documented acceptance) | 2026-07-05 |
| AR-254-02 | T-254-05 | `pub mod classes` exposes constant strings only; structural fix for dead_code under `-D warnings` (D-16); pre-1.0, no semver guarantee | gsd-secure-phase (plan-documented acceptance) | 2026-07-05 |
| AR-254-03 | T-254-06 | Lint is a dev/CI-time diagnostic pass with no execution of spec-supplied strings; misleading findings are the worst case | gsd-secure-phase (plan-documented acceptance) | 2026-07-05 |
| AR-254-04 | T-254-07 | patterns.md and RULE_COMPONENTS are public documentation surfaces by design | gsd-secure-phase (plan-documented acceptance) | 2026-07-05 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-05 | 7 | 7 | 0 | gsd-secure-phase (orchestrator evidence pass — mitigations verified by direct code/test inspection; no auditor spawn needed for a contracts-only phase) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-05
