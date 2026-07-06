---
phase: 255
slug: pos-runtime-modules-double-submit-protection
status: secured
threats_open: 0
asvs_level: 1
created: 2026-07-05
---

# Phase 255 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| spec author → render (server) | Untrusted `TileProps` values (`name`, `categories`, `item_id`) cross into emitted HTML attributes at render time | Attribute values (XSS surface) |
| browser → runtime JS | User taps drive hidden-input mutation and client-side visibility; all state is presentational | Form field values, filter state |
| browser → server (form POST) | The single confirm POST carries accumulated quantities; double-submit and replay surface | Order quantities, idempotency key |
| spec author → lint (pre-render) | Raw spec JSON is linted; diagnostics-only, no execution | Spec JSON |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-255-01 | Information disclosure / XSS | `render_tile` `data-filter-tokens` | mitigate | `html_escape` on joined token list (atoms.rs:1383-1392); `tile_escapes_categories` regression test | closed |
| T-255-02 | Tampering (serde) | `SelectionPanelProps` field removal | accept | No `deny_unknown_fields` on component structs — legacy specs with removed keys deserialize silently, no data exposure | closed |
| T-255-03 | Tampering | `design/rules.rs` register-* rules | accept | Diagnostics-only, pure, pre-expansion; rename changes no evaluation logic | closed |
| T-255-04 | Denial of service | runtime bundle assembly (`mod.rs`) | accept | Static `LazyLock<String>`; both drift-list tests enforce completeness | closed |
| T-255-05 | Spoofing / integrity | ferro-mcp catalog mirror | mitigate | `test_all_components_present` asserts `"Tile"` + register-* ids against live catalog; count 47 unchanged | closed |
| T-255-06 | Repudiation | docs migration note | accept | Descriptive prose only; SC-0 global grep zero hits across all four trees | closed |
| T-255-07 | Information disclosure / XSS | `render_tile` `data-filter-text` | mitigate | `html_escape(&props.name)` (atoms.rs:1370) unconditionally; `tile_escapes_filter_text` asserts `&quot;`/`&lt;` | closed |
| T-255-08 | Elevation of privilege | `filters.rs` client-side visibility | accept | `style.display` only; tiles server-rendered from session-authorized data; server never trusts filter state | closed |
| T-255-09 | Tampering (money) | `numpad.rs` price mode | mitigate | Integer-cents hidden-field contract documented in module + JS comments; display formatting presentational only; server re-validates; selector sanitization (WR-03 fix) | closed |
| T-255-10 | Tampering / DoS | double-submit guard (`form_guards.rs`) | mitigate | Submit-event-bound latch with `defaultPrevented` check (WR-02 fix) + `btn._submitted` + `pageshow` bfcache reset; form binding fixed (WR-01) and verified by live browser UAT (255-HUMAN-UAT.md: one POST, Enter-key blocked, bfcache reset) | closed |
| T-255-11 | Elevation of privilege | client-guard bypass (JS-off / crafted POST) | transfer | Server-side `dispatch_write` idempotency hook keyed on `(tenant_id, idempotency_key)` — documented in write-kernel.md "Double-submit protection for forms"; client guard explicitly framed as UX affordance, not a security control | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party / other layer)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-255-01 | T-255-02 | Component structs intentionally tolerate unknown serde fields (wire-compat posture); removed keys carry no sensitive data | operator (plan threat model) | 2026-07-05 |
| AR-255-02 | T-255-03 | Lint rules never execute spec content; rename is id-level only | operator (plan threat model) | 2026-07-05 |
| AR-255-03 | T-255-04 | Bundle is static; no dynamic assembly path exists | operator (plan threat model) | 2026-07-05 |
| AR-255-04 | T-255-06 | Migration documentation is prose; no executable surface | operator (plan threat model) | 2026-07-05 |
| AR-255-05 | T-255-08 | Client-side filtering is a UX affordance over already-authorized data; authorization stays server-side | operator (plan threat model) | 2026-07-05 |

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-05 | 11 | 11 | 0 | gsd-security-auditor (post-execution, post-review-fix; includes live-UAT evidence for T-255-10) |

Audit notes: all three code-review warnings (WR-01 form binding, WR-02 `defaultPrevented`,
WR-03 dispatcher try/catch + selector sanitization) were fixed before this audit and
directly strengthen T-255-09/10 closures. All SUMMARY.md threat flags map to registered
threat IDs — no unregistered attack surface.
