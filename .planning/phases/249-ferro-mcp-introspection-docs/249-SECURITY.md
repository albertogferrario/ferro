---
phase: 249
slug: ferro-mcp-introspection-docs
status: verified
threats_open: 0
threats_total: 5
asvs_level: 1
created: 2026-08-15
audited: 2026-08-15
auditor: gsd-security-auditor
verdict: SECURED
---

# Phase 249 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

Scope: two plans — 249-01 (ferro-mcp offload introspection, a read-only static
source parse plus additive `list_services` output) and 249-02 (the canonical
`docs/src/features/offload.md` documentation page). No new network endpoint,
auth surface, secret, or untrusted-input boundary is introduced by either plan.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| local filesystem → ferro-mcp static parser | Parser reads `{project_root}/src/**/*.rs` — already under the trusted project root read by the pre-existing `scan_services_from_files` walk. No new path surface. | Developer's own Rust source text (read-only) |
| ferro-mcp → MCP client (agent) | `list_services` returns additive JSON derived from source the agent can already read directly. | Method names, queue names, Rust type strings |
| repository docs → public reader | `docs/src/features/offload.md` is a public artifact; the boundary is editorial (neutral voice, code-accurate claims). | Public documentation prose |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-249-01 | Tampering | static parser reading local `src/**/*.rs` | accept | Parser confined to `project_root.join("src")` (list_services.rs:349); same WalkDir filter as the pre-existing `scan_services_from_files`; no write, no exec, no path outside the trusted root. `generation_context.rs:23` carries a read-only `&'static str` field. | closed |
| T-249-02 | Information disclosure | `methods` array in `list_services` output | accept | `OffloadParam { name: String, rust_type: String }` (list_services.rs:31-35) and `OffloadableMethod { name, queue, params }` (list_services.rs:37-45) carry only method/queue/type strings already present in readable source. No secret, PII, or credential type. Tool description confirmed at service.rs:606,610. | closed |
| T-249-03 | Denial of service | pathological source triggering unbounded `FnCollecting` accumulation | mitigate | `scan_offload_methods_from_files` iterates via `for line in content.lines()` (list_services.rs:383) — bounded by the file's finite line count. `FnCollecting` accumulates into `buf` per line and terminates on `paren_depth < 0` (list_services.rs:520); no independent unbounded loop. Both `execute()` call sites present (grep count == 2). | closed |
| T-249-04 | Information disclosure | docs prose (public repository artifact) | mitigate | `grep -Eic "killer feature\|the bet\|load-bearing\|we accept that\|forcing function" docs/src/features/offload.md` → 0 matches. Deferred directions framed as future work under `## Non-goals (2.0 direction)`, not internal-strategy commitments. | closed |
| T-249-05 | Tampering | doc claims vs shipped code | accept | Two of four ASSUMED facts re-grepped against code at audit: A1 confirmed (`enqueue_and_mark_pending` framework/src/offload.rs:350, `read_result` :197, `read_result_redacted` :225, `resolve` :400); A2 confirmed (`CreateProjectionSnapshotsTable` ferro-projection/src/lib.rs:87). Residual stale-doc drift accepted as normal maintenance risk. | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| AR-249-01 | T-249-01 | Read-only local source walk within the same trust boundary as the existing static parser. Developer's own source; no new capability introduced. | gsd-secure-phase | 2026-08-15 |
| AR-249-02 | T-249-02 | Exposed data (method names, queue names, Rust type strings) is already present in readable source files; semantically equivalent to the agent reading the trait directly. | gsd-secure-phase | 2026-08-15 |
| AR-249-05 | T-249-05 | Docs describe a shipped surface verified against code at authoring time. Post-ship code changes may introduce drift; this is normal documentation maintenance, not a security threat. | gsd-secure-phase | 2026-08-15 |

*Accepted risks do not resurface in future audit runs.*

---

## Unregistered Flags

None. Both SUMMARY files (`249-01-SUMMARY.md`, `249-02-SUMMARY.md`) report "None"
under Threat Flags / Threat Surface Scan. No attack surface beyond the
registered threats was identified by the executor.

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-08-15 | 5 | 5 | 0 | gsd-security-auditor (State B — initial audit) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-08-15
