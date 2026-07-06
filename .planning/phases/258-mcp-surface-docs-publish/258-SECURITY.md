---
phase: 258
slug: mcp-surface-docs-publish
status: verified
threats_open: 0
asvs_level: 1
created: 2026-07-06
---

# Phase 258 — Security

> Per-phase security contract: threat register, accepted risks, and audit trail.

---

## Trust Boundaries

| Boundary | Description | Data Crossing |
|----------|-------------|---------------|
| ferro-mcp output → agent authoring | generation_context / json_ui_catalog are read-only advisory context an agent consumes to build a spec; no new inputs, no auth path, no execution crosses here | Component/attribute/rule guidance (public, derived from in-crate registries) |
| docs/src → human/agent authors | mdBook documentation consumed by authors composing specs; incorrect guidance propagates into consumer code | Public documentation |
| local workspace → crates.io / GitHub (CI publish) | The push exposes code to public registries and is irreversible once published; a supply-chain boundary | Full source tree (public) |

---

## Threat Register

| Threat ID | Category | Component | Disposition | Mitigation | Status |
|-----------|----------|-----------|-------------|------------|--------|
| T-258-01 | Information Disclosure | register_composition guidance asserts a wrong attribute/rule/component → agent builds a broken or insecure register (double-submit, wrong form-state contract) | mitigate | `register_composition_drift_guard()` (ferro-mcp/src/tools/generation_context.rs:558) asserts component names against `global_catalog()`, lint ids against `design::rules()`, and all 13 `REGISTER_DATA_ATTRIBUTES` against `FERRO_RUNTIME_JS` (extended by WR-02 fix 208cc3aa); `data-disable-on-submit` explicitly asserted | closed |
| T-258-02 | Tampering | BUILDER_API / RULE_COMPONENTS drift from the actual public API, misleading the agent | mitigate | `builder_api_mentions_fill_viewport_and_each()` (ferro-mcp/src/tools/json_ui_catalog.rs:595) plus the existing 3-direction `design_system_component_guidance_drift_guarded`; rule-id assertions pin expected literals, not derived output (WR-03 fix 305a509e) | closed |
| T-258-03 | Spoofing | No identity/authn surface added — MCP output derived from in-crate registries only, no external data source | accept | Documented in Accepted Risks Log (R-258-01) | closed |
| T-258-04 | Tampering | Docs misdocument the double-submit / idempotency pattern or the form-state single-source contract → author ships a double-submittable confirm or a second source of truth | mitigate | docs/src/json-ui/components.md:1501 states the `disable_on_submit` → `data-disable-on-submit` guard, the `framework::write` idempotency-hook cross-link, and "the panel is not a second source of truth — the hidden inputs in the Form are" | closed |
| T-258-05 | Information Disclosure | Props tables drift from the actual component API, leading authors to wrong field names/types | mitigate | Prop rows copied from verified ground truth (component.rs:1412–1529 via RESEARCH); `target_field`/`form_id` present in components.md; `mdbook build docs` exits 0 (258-VERIFICATION.md, re-run in verification) | closed |
| T-258-06 | Repudiation | A new SUMMARY.md page with a missing file breaks the docs build (`create-missing = false`) | mitigate | D-08 upheld: existing pages extended only; docs/src/SUMMARY.md last touched in phase 253 (git log confirms unchanged); `mdbook build docs` exits 0 | closed |
| T-258-07 | Tampering | Publishing a broken build to crates.io | mitigate | CI Publish run 28808914072: Test job green (13m1s, CI-exact fmt/clippy/test/doc gate), all five publish waves green; publish gated behind operator checkpoint (Task 2, approved) | closed |
| T-258-08 | Repudiation | Publishing the wrong tree or version — stale local refs, unreviewed rider commits swept in | mitigate | Version verified against crates.io API (ferro-rs 0.2.89, ferro-payments 0.1.6 — both confirmed live); tag `v0.2.89` verified on remote via gh api; master fast-forwarded `--ff-only` with HEAD=master asserted from main repo root; pre-publish checklist enumerated the ferro-payments 0.1.6 rider | closed |
| T-258-09 | Elevation of Privilege | Cross-repo edits to the gestiscilo consumer tree from this session | mitigate | D-17 upheld: handoff is a brief embedded in 258-03-SUMMARY.md only; no consumer-repo or consumer-planning edits | closed |
| T-258-10 | Information Disclosure | cargo-deny / license or advisory regressions shipped in the publish | accept | Documented in Accepted Risks Log (R-258-02) | closed |
| T-258-11 | Tampering | Stray untracked planning artifacts committed into the publish commit | mitigate | `git show --stat 34279ca7` lists exactly Cargo.toml + Cargo.lock — no stray artifacts; specific-file staging used, never `git add -A` | closed |

*Status: open · closed*
*Disposition: mitigate (implementation required) · accept (documented risk) · transfer (third-party)*

---

## Accepted Risks Log

| Risk ID | Threat Ref | Rationale | Accepted By | Date |
|---------|------------|-----------|-------------|------|
| R-258-01 | T-258-03 | No identity/authn surface added this phase; MCP output is derived from in-crate registries only, no external data source introduced | gsd-secure-phase (plan disposition) | 2026-07-06 |
| R-258-02 | T-258-10 | CI runs cargo-deny; no new external crates this phase (docs + MCP output additions + version bump only), so the advisory/license surface is unchanged; `ferro-a2ui` stays `publish = false` | gsd-secure-phase (plan disposition) | 2026-07-06 |

*Accepted risks do not resurface in future audit runs.*

---

## Security Audit Trail

| Audit Date | Threats Total | Closed | Open | Run By |
|------------|---------------|--------|------|--------|
| 2026-07-06 | 11 | 11 | 0 | gsd-secure-phase (State B, from artifacts) |

---

## Sign-Off

- [x] All threats have a disposition (mitigate / accept / transfer)
- [x] Accepted risks documented in Accepted Risks Log
- [x] `threats_open: 0` confirmed
- [x] `status: verified` set in frontmatter

**Approval:** verified 2026-07-06
