# Phase 253: MCP surface + docs + publish - Context

**Gathered:** 2026-07-04 (auto mode — recommended defaults selected, logged in 253-DISCUSSION-LOG.md)
**Status:** Ready for planning

<domain>
## Phase Boundary

Close the agent-authoring loop and ship the v16.5 milestone: expose `design_lint`
through ferro-mcp, extend `json_ui_catalog` with the canonical variant vocabulary
and per-component design guidance, extend `generation_context` with a design-system
summary (tokens + per-intent pattern expectations), write the
`docs/src/design-system/` chapter, and perform the single crates.io publish that
unblocks the consumer adoption phase (gestiscilo Phase 232). Requirements: DS-07,
DS-08.

**Killer feature framing:** this phase completes "design system enforced at the
agent-authoring boundary" — an agent reads the system through ferro-mcp
(`generation_context` + `json_ui_catalog`), authors a spec, and validates it with
`design_lint` inside the same session, before any human review. The MCP tool is
what turns Phase 252's rule engine from a CI gate into an authoring-time loop.

Out of scope: new rules or rule-engine changes (252 owns the engine; only bug-fix
level touches allowed), new components (count stays 47), new tokens (250 closed the
vocabulary at 30), the gestiscilo adoption work itself (consumer repo, separate
phase).

</domain>

<decisions>
## Implementation Decisions

### design_lint MCP tool (DS-07)
- **D-01:** Input contract: `spec_json` (inline JSON string) OR `path` (single spec
  file) — exactly one required, per spec §4 ("inline JSON or path") and §7 (tests
  for both input modes). Directory sweeps remain the CLI's job; the MCP tool is the
  in-session author→validate loop for one spec.
- **D-02:** Output reuses the `Finding`/`Severity` serialization from 252 D-11 —
  the same shape the CLI `--json` emits (findings + summary/has_warning fields).
  One serialization across CLI and MCP; no MCP-only envelope invention.
- **D-03:** Implementation is in-process: ferro-mcp already depends on
  ferro-json-ui **with the `projections` feature** (`ferro-mcp/Cargo.toml:24`), so
  the tool calls `ferro_json_ui::design::lint` directly. Registration follows the
  established `service.rs` `#[tool]` + `tools/design_lint.rs::execute()` pattern
  (closest analog: `json_ui_validate_spec`, `service.rs:1405`).
- **D-04:** `design_lint` is lint-only — it does NOT run catalog validation (252
  D-scoped separation: catalog validation = hard errors, design lint =
  diagnostics). A spec that fails `Spec` parse returns a parse diagnostic inside
  the same findings envelope (the CLI WR-03 file-diagnostic posture), never a tool
  error.

### json_ui_catalog + generation_context extensions (DS-07)
- **D-05:** The canonical variant vocabulary in `json_ui_catalog` is **derived from
  the canonical enums** (Variant/Tone/Size/CardAppearance from Phase 251) — not a
  hand-listed table. Per-component design guidance is derived from
  `design::rules()` metadata where a rule references the component; any small
  static supplement must carry a drift test tying it to the registry/enum set
  (252 D-10 single-registry philosophy). All new catalog fields are additive —
  existing output shape stays backward-compatible.
- **D-06:** `generation_context` design-system summary contains: (a) the token v2
  vocabulary (30 slots with one-line purpose each, sourced from ferro-theme's
  token constants), (b) per-intent pattern expectations derived from the rule
  registry (rule id + title + rationale grouped by intent), (c) the canonical
  variant/tone/size value lists. Keep it compact — ids and one-liners, with a
  pointer to `docs/src/design-system/` for depth; generation_context is inline
  agent context, not a manual.
- **D-07:** Component count stays 47 — the ferro-mcp documented mirror assertion
  (`json_ui_catalog.rs:294`) is untouched. Grep for mirrored assertions before the
  gate anyway (established practice). No `ferro-base.css` regeneration expected
  (no class changes this phase).

### docs/src/design-system/ chapter (DS-08)
- **D-08:** Five pages + SUMMARY.md section: `principles.md` (semantic tokens,
  intent-keyed patterns, lint-as-diagnostics), `tokens.md` (token v2 reference),
  `variants.md` (canonical vocabulary), `patterns.md` (pattern catalog: per rule —
  rationale, violating/conforming example, how to `allow`), `linting.md` (CLI +
  MCP guide). Cross-link `features/themes.md` (theme authoring recipe stays there,
  from 250) and `json-ui/components.md` (251 D-17 migration table stays there —
  link it, never duplicate it).
- **D-09:** The pattern catalog page is hand-written prose but **drift-guarded**: a
  test asserts every rule id from `design::rules()` appears in `patterns.md` and
  every documented rule id exists in the registry — mirroring the count-drift-guard
  practice. Rationale text should come from the registry's `rationale` field so
  prose and machine metadata cannot diverge silently.
- **D-10:** Docs voice: neutral product documentation. No "v2 vs legacy" framing
  beyond the token-version labels already established
  (`feedback_json_ui_naming`); no internal strategy language
  (repository-documents-must-read-as-neutral rule).

### Publish (DS-08)
- **D-11:** Single publish at phase end. Local master already carries unpushed
  bumps through 0.2.83 (crates.io is at 0.2.80): land all 253 code, run the
  CI-exact gate, do ONE final workspace bump as the publish commit, push master
  via the gh HTTPS credential helper → CI publishes. Verify the result via
  crates.io / gh API, never stale `origin/master` refs (recurrent local-refs lie;
  fix with `git update-ref` after a verified push).
- **D-12:** CI-exact gate before the publish push: `cargo fmt --all -- --check`,
  `cargo clippy --all --all-targets --all-features -- -D warnings`,
  `cargo test --all-features`, plus the Docs build (`cargo doc` `-D warnings`) and
  cargo-deny awareness — CI's matrix is wider than the local three-command gate
  (`feedback_ci_matrix_wider_than_local_gate`). Re-run fmt after ANY hand-edit.
- **D-13:** ferro-payments is independently versioned (0.1.3) and untouched by
  v16.5 — no ferro-payments bump. No new crates this milestone, so no
  publish.yml wave changes and no publish-new token bootstrap needed.
- **D-14:** The publish step is operator-gated (Phase 236 practice): present the
  pre-publish UAT checklist at that gate. Fold the two open v16.5 human items into
  it — 252's human-readable CLI output check (`ferro design:lint app/src/views`)
  and 251's suggested pixel-level visual pass on the refreshed theme.
- **D-15:** Fold 252's two deferred info-findings as pre-publish cleanup (both
  one-liners; publish freezes the UX for the consumer): IN-01 remove the dead
  `"Textarea"` entry from `FIELD_TYPES` (`design/rules.rs:298`); IN-02 fix the
  misleading "No findings — all specs are clean" when zero files were linted
  (`commands/design_lint.rs`).
- **D-16:** Publishing unblocks gestiscilo Phase 232. Cross-repo handoff is a
  brief only (`feedback_cross_repo_phase_split`) — never edit the consumer tree or
  its planning from this session.

### Claude's Discretion
- Exact field names and struct layout for the new catalog / generation_context
  fields; whether per-intent expectations embed rationale verbatim or trimmed.
- Whether `design_lint` returns rule metadata (title/rationale) inline per finding
  or only ids (consistency with CLI `--json` wins ties).
- Doc page ordering, titles, and intra-chapter navigation within the five-section
  requirement.
- Whether the docs drift test lives in ferro-json-ui or a workspace-level test —
  pick the home that matches existing doc-drift guards.
- The final version number (next patch after whatever master carries when the
  publish commit is cut).

### Folded Todos
None — no pending todos matched this phase (`todo match-phase 253` returned 0).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Design spec (anchor — source of truth for this milestone)
- `docs/superpowers/specs/2026-07-03-json-ui-design-system-design.md` §4
  (surfaces: ferro-mcp tool contract, catalog/generation-context extensions, docs
  chapter outline), §6 (gestiscilo adoption — what the publish must unblock), §7
  (testing: MCP tool tests for inline + path input; CI-exact gate), §8 (non-goals).

### Prior phase decisions (what this phase consumes)
- `.planning/phases/252-design-module-lint-cli/252-CONTEXT.md` — D-10 (public
  rule registry exists FOR this phase's docs+MCP derivation), D-11 (shared
  `Finding` serialization = the MCP output contract), D-05 (CLI `--json` shape is
  the stable consumer contract).
- `.planning/phases/252-design-module-lint-cli/252-VERIFICATION.md` — deferred
  IN-01/IN-02 (folded here as D-15), the open human CLI-output check (folded into
  D-14), and the pre-existing flaky `serve.rs` test documented in
  `.planning/phases/252-design-module-lint-cli/deferred-items.md` (NOT a 253 gap —
  don't chase it at the gate).
- `.planning/phases/251-component-variant-discipline-interactive-state-pass/251-CONTEXT.md`
  — D-17 migration table home (`docs/src/json-ui/components.md`) and the canonical
  enum set the catalog vocabulary derives from.

### Planning
- `.planning/ROADMAP.md` — v16.5 section, Phase 253 details (goal, success
  criteria 1–4).
- `.planning/REQUIREMENTS.md` — DS-07, DS-08 (v16.5 Agent Surface & Docs section).

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `ferro-json-ui/src/design/` (mod.rs, rules.rs, types.rs, infer.rs) —
  `design::rules() -> &'static [DesignRule]` (mod.rs:50) with public
  id/title/rationale/intents metadata; `Finding`/`Severity` (types.rs:13,26)
  already derive Serialize — the MCP tool and docs derivation consume these as-is.
- `ferro-mcp/src/tools/json_ui_validate_spec.rs` + `service.rs:1405` — the
  closest tool analog (inline `spec_json` input, `execute()` in tools/, `#[tool]`
  registration in service.rs). `design_lint` follows this shape plus a `path`
  alternative.
- `ferro-mcp/src/tools/json_ui_catalog.rs` — `execute(component)` builds per-
  component entries; `derive_variants()` (line 222) already extracts enum variants
  from Props schemas; the 47-count mirror assertion is at line 294.
- `ferro-mcp/src/tools/generation_context.rs` — static `execute() ->
  GenerationContext` struct; the design-system summary is a new field group on it.
- `ferro-cli/src/commands/design_lint.rs` — the CLI `--json` envelope and
  file-diagnostic (WR-03) posture the MCP tool mirrors; IN-02 fix lands here.
- `docs/src/SUMMARY.md:63-73` — the JSON-UI chapter block the new
  `design-system/` section slots next to.

### Established Patterns
- ferro-mcp depends on ferro-json-ui with `features = ["projections"]` — the
  design module and intent labels are fully available in-process (no
  feature-gating gymnastics needed, unlike 252's D-07 CLI constraint).
- Doc/count drift guards: single guard in the owning crate, ferro-mcp stays a
  documented mirror (memory: builtin-component count consolidation). The D-09
  docs↔registry drift test follows this philosophy.
- Publish flow: version bump commit → push master → CI publish.yml (waves) →
  GitHub Release + binaries + brew tap self-bump. Workspace at 0.2.83 local /
  0.2.80 crates.io; bumps 0.2.81–0.2.83 are unpushed local commits.
- CI clippy/test run `--all-features`; local convenience gates miss
  `--all-features`-only failures (`feedback_ci_clippy_command_match`).

### Integration Points
- `ferro-mcp/src/service.rs` — tool registration for `design_lint`; ferro-mcp
  tool descriptions are framework surface held to the Rust-API quality bar
  (project CLAUDE.md).
- `docs/src/SUMMARY.md` — new chapter registration.
- gestiscilo Phase 232 (consumer repo) pins the published release and consumes:
  the CLI `--deny` gate, the `--json` shape, the MCP `design_lint` tool, and the
  migration table. All are public contracts the moment this publish lands.

</code_context>

<specifics>
## Specific Ideas

- The whole phase is a derivation exercise: catalog vocabulary from the canonical
  enums, per-intent expectations and the pattern-catalog docs from
  `design::rules()`, token reference from ferro-theme constants. Nothing
  hand-duplicated; every derived surface drift-guarded ("structural guarantees
  over one-off fixes").
- MCP tool descriptions and `generation_context` quality are the product here —
  write them for an agent authoring its first conformant spec on the first pass,
  not as API reference boilerplate.
- The publish is the milestone's exit: success criterion 4 explicitly includes
  "the consumer adoption phase (gestiscilo Phase 232) is unblocked".

</specifics>

<deferred>
## Deferred Ideas

- gestiscilo Phase 232 reference-case adoption (68-spec sweep, `--deny` CI gate,
  FRICTION.md) — consumer repo, gated on this publish; handoff brief only.
- `/gsd-complete-milestone` archival for v16.0/v16.1/v16.2/v16.3 (and v16.5 after
  this phase) — milestone bookkeeping, not phase work.
- CSS-hygiene lint (dead utilities in generated `ferro-base.css` from negative
  test assertions) — carried from 252 deferral; revisit only if a CSS-hygiene rule
  category materializes.
- OQ-3 `dot_colors` raw-Tailwind rule — not implemented in 252's 10-rule registry;
  stays with the gestiscilo FRICTION.md loop to prioritize.
- v16.4 Work Distribution (Phases 244–249) — independent milestone, either order.
- Pre-existing flaky `serve.rs` PGID test — documented in 252's
  deferred-items.md; fix is a standalone quick task, not a 253 gate item.

</deferred>

---

*Phase: 253-mcp-surface-docs-publish*
*Context gathered: 2026-07-04*
