# Requirements: v13.0 Compressive Validation

**Milestone goal:** Validate the projection / intent abstraction empirically — the first slice of the v13.0 "Road to v1.0" program. Targets the compressive dimension (substance-first priority #1) and v1.0 criterion #2: *projection / intent is validated through real applications and a synthetic catalog of canonical app classes.*

**Scope:** Validation and measurement against ferro's own projection/intent system. No new published crates; no changes to the seven-intent vocabulary (`ferro-projections/src/intent.rs`) in this milestone. Phase numbering continues from 206 (v13.0 starts at Phase 207).

**The honesty requirement (applies to every COMP phase):** Validation must be able to *fail* and surface real weaknesses — a weakness in any beauty dimension is a v1.0 blocker. Every phase MUST name an adversarial input/fixture and include a "discovered weaknesses" section in its verification. A phase that finds nothing wrong is a red flag, not a success.

## v13.0 Requirements

### Compressive Validation

- [ ] **COMP-01**: A real application (`gestiscilo`) is partially migrated to projection-driven rendering — **Slice A**: three entities spanning the Browse, Process, and Summarize intents, migrated one-per-merge with render equivalence checked against the existing views, and a single ferro publish at the end of the slice. This is the first real-world validation signal for the projection/intent abstraction. (Full gestiscilo migration — 130 views, 69 models — is explicitly out of scope for v13.0.)
- [x] **COMP-02**: A synthetic catalog of canonical app classes covering the seven structural intents exists with a regression harness that runs on every `derive_intents()` / projection change. The harness asserts **structural invariants** (e.g. the derived primary intent and at least one key signal per fixture; a Browse projection renders a table with the correct column count) rather than byte-for-byte renderer snapshots, and includes at least one fixture with competing signals proving the intended intent wins under competition.
- [ ] **COMP-03**: An agent-success-rate harness measures whether an agent reading `ferro-mcp` introspection can produce a working projection from a natural-language description. Pass criteria are **multi-tier** (structural validity → intent coverage → functional completeness → checkpoint pass) and **stated before any runs are collected**; each task runs ≥3 trials; a baseline (model version, prompt version, per-tier pass rates) is committed. The corpus spans all seven intents. The harness drives ferro-mcp developer tools as an in-process client (not `ferro-mcp-server`), and guards against training-data contamination.
- [ ] **COMP-04**: A time-to-working-app benchmark measures `cargo new` → a running service with authentication, three entity types, and one background job. The published number includes at least one **cold-cache run** (no warm Cargo cache / pre-installed toolchain); the measurement apparatus is committed as a document. The benchmark target is gated (`FERRO_BENCH=1`) to avoid exhausting CI disk.
- [ ] **COMP-05**: An intent-vocabulary cross-modality sketch takes one intent (e.g. `Process`) and expresses the same feature as a mobile flow, a voice interaction, and a CLI command, producing a **document** that analyzes whether the seven-intent vocabulary survives non-visual rendering. The deliverable is a v14.0 planning input only — it MUST NOT modify `intent.rs` or any renderer in v13.0. v14.0 Channel Projection depends on this analysis.

## Future Requirements (deferred)

- **Rest of the Road to v1.0 program** — the operational (OPER-01..07), conceptual (CONC-01..04, incl. crate-consolidation audit + ServiceDef derivation bridge), and aesthetic (AEST-01..04) dimensions are subsequent v13.x milestones, prioritized after the compressive validation establishes baseline signal.
- **Full gestiscilo migration** — beyond Slice A, migrating the remaining views/models once the abstraction is validated.
- **Intent vocabulary revision** — if COMP-05 reveals the seven intents need reshaping for non-visual rendering, that revision is a v14.0 research outcome (tracked as CHAN-05), not a v13.0 change.

## Out of Scope

| Item | Reason |
|------|--------|
| Changing `ferro-projections/src/intent.rs` (the 7 intents) | COMP-05 is a probe, not a build; any revision cascades through 5+ crates and is a v14.0 decision |
| New published crates | All five COMP artifacts fit in existing crate `tests/` dirs or `pub(crate)` sketch modules |
| rmcp upgrade (≥1.5) | `rmcp 0.12`'s `transport-async-rw` in-process transport covers COMP-03; upgrading is a breaking change across 3 crates |
| `hyperfine` or external benchmark binaries | Violates the no-external-build-tooling constraint; use `criterion` |
| Full gestiscilo migration | Months of cross-repo work; Slice A provides the v1.0 validation signal |
| Non-visual renderer *implementations* | v14.0 Channel Projection direction; COMP-05 only sketches |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| COMP-01 | Phase 209 | Pending |
| COMP-02 | Phase 207 | Complete |
| COMP-03 | Phase 210 | Pending |
| COMP-04 | Phase 211 | Pending |
| COMP-05 | Phase 208 | Pending |
